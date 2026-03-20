mod d1_repo;

use d1_repo::D1Repo;
use openwok_core::money::Money;
use openwok_core::order::OrderStatus;
use openwok_core::repo::{
    CreateCourierRequest, CreateMenuItemRequest, CreateOrderItemRequest, CreateOrderRequest,
    CreateRestaurantRequest,
};
use openwok_core::types::{
    CourierId, CreatePaymentRequest, CreateUserRequest, MenuItemId, OrderId, PaymentStatus,
    RestaurantId, UpdateMenuItemRequest, UpdatePaymentStatusRequest, UpdateRestaurantRequest,
    UserRole, ZoneId,
};
use serde::Deserialize;
use worker::*;

// ── Request DTOs ───────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CreateRestaurantReq {
    name: String,
    zone_id: String,
    menu: Vec<CreateMenuItemReq>,
}

#[derive(Deserialize)]
struct CreateMenuItemReq {
    name: String,
    price: String,
}

#[derive(Deserialize)]
struct CreateOrderReq {
    restaurant_id: String,
    items: Vec<CreateOrderItemReq>,
    customer_address: String,
    zone_id: String,
    delivery_fee: String,
    tip: String,
    local_ops_fee: String,
    origin_url: Option<String>,
}

#[derive(Deserialize)]
struct CreateOrderItemReq {
    menu_item_id: String,
    name: String,
    quantity: u32,
    unit_price: String,
}

#[derive(Deserialize)]
struct TransitionReq {
    status: String,
}

#[derive(Deserialize)]
struct CreateCourierReq {
    name: String,
    zone_id: String,
}

#[derive(Deserialize)]
struct SetAvailableReq {
    available: bool,
}

#[derive(Deserialize)]
struct UpdateRestaurantReq {
    name: Option<String>,
    description: Option<String>,
    address: Option<String>,
    phone: Option<String>,
}

#[derive(Deserialize)]
struct ToggleActiveReq {
    active: bool,
}

#[derive(Deserialize)]
struct UpdateMenuItemReq {
    name: Option<String>,
    price: Option<String>,
}

// ── Auth DTOs ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AuthCallbackReq {
    access_token: String,
}

// ── JWT Verification (manual HS256 — no ring dependency) ──────────────

fn verify_jwt(token: &str, secret: &str) -> Result<(String, Option<String>)> {
    use base64::Engine;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(Error::RustError("invalid JWT format".into()));
    }

    // Verify HMAC-SHA256 signature
    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let signature = engine
        .decode(parts[2])
        .map_err(|_| Error::RustError("invalid JWT signature encoding".into()))?;

    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|_| Error::RustError("HMAC key error".into()))?;
    mac.update(signing_input.as_bytes());
    mac.verify_slice(&signature)
        .map_err(|_| Error::RustError("invalid JWT signature".into()))?;

    // Decode payload
    let payload_bytes = engine
        .decode(parts[1])
        .map_err(|_| Error::RustError("invalid JWT payload encoding".into()))?;
    let claims: serde_json::Value = serde_json::from_slice(&payload_bytes)
        .map_err(|_| Error::RustError("invalid JWT payload JSON".into()))?;

    // Check expiration
    if let Some(exp) = claims["exp"].as_u64() {
        let now = (js_sys::Date::now() / 1000.0) as u64;
        if now > exp {
            return Err(Error::RustError("JWT expired".into()));
        }
    }

    let sub = claims["sub"]
        .as_str()
        .ok_or_else(|| Error::RustError("missing sub claim".into()))?
        .to_string();
    let email = claims["email"].as_str().map(|s| s.to_string());

    Ok((sub, email))
}

/// Extract and verify JWT from Authorization: Bearer header.
fn extract_auth(req: &Request, env: &Env) -> Result<(String, Option<String>)> {
    let secret = env
        .secret("SUPABASE_JWT_SECRET")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| "super-secret-jwt-token-for-testing-only".into());

    let auth_header = req
        .headers()
        .get("Authorization")
        .map_err(|_| Error::RustError("missing header".into()))?
        .ok_or_else(|| Error::RustError("missing Authorization header".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| Error::RustError("invalid Authorization format".into()))?;

    verify_jwt(token, &secret)
}

// ── Helpers ────────────────────────────────────────────────────────────

fn parse_uuid(s: &str) -> uuid::Uuid {
    uuid::Uuid::parse_str(s).unwrap_or_else(|_| uuid::Uuid::nil())
}

fn parse_status(s: &str) -> Option<OrderStatus> {
    match s {
        "Created" => Some(OrderStatus::Created),
        "Confirmed" => Some(OrderStatus::Confirmed),
        "Preparing" => Some(OrderStatus::Preparing),
        "ReadyForPickup" => Some(OrderStatus::ReadyForPickup),
        "InDelivery" => Some(OrderStatus::InDelivery),
        "Delivered" => Some(OrderStatus::Delivered),
        "Cancelled" => Some(OrderStatus::Cancelled),
        _ => None,
    }
}

fn json_ok<T: serde::Serialize>(data: &T, status: u16) -> Result<Response> {
    let body = serde_json::to_string(data).map_err(|e| Error::RustError(e.to_string()))?;
    let mut resp = Response::ok(body)?;
    resp = resp.with_status(status);
    resp.headers_mut().set("Content-Type", "application/json")?;
    Ok(resp)
}

/// Convert auth errors to 401 responses.
fn auth_err_to_response(e: Error) -> Result<Response> {
    Response::error(format!("Unauthorized: {e}"), 401)
}

/// Convert Money amount to Stripe cents (integer).
fn to_cents(money: &Money) -> i64 {
    use rust_decimal::prelude::ToPrimitive;
    (money.amount() * rust_decimal::Decimal::from(100))
        .round()
        .to_i64()
        .unwrap_or(0)
}

/// Get user and verify not blocked. Returns error Response if user not found or blocked.
async fn require_active_user(
    repo: &D1Repo,
    sub: &str,
) -> std::result::Result<openwok_core::types::User, Result<Response>> {
    let user = repo
        .get_user_by_supabase_id(sub)
        .await
        .map_err(|_| Response::error("user not found", 401))?;
    if user.blocked {
        return Err(Response::error("user is blocked", 403));
    }
    Ok(user)
}

fn repo_err_to_response(e: openwok_core::repo::RepoError) -> Result<Response> {
    use openwok_core::repo::RepoError;
    match &e {
        RepoError::NotFound => Response::error(e.to_string(), 404),
        RepoError::InvalidTransition(_) => Response::error(e.to_string(), 400),
        RepoError::Conflict(_) => Response::error(e.to_string(), 400),
        RepoError::Internal(_) => Response::error(e.to_string(), 500),
    }
}

// ── Entry point ────────────────────────────────────────────────────────

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Non-API routes → serve from static assets (SPA fallback via not_found_handling)
    let path = req.url()?.path().to_string();
    if !path.starts_with("/api/") {
        let assets = env.assets("ASSETS")?;
        return assets.fetch_request(req).await;
    }

    let router = Router::new();

    router
        .get_async("/api/health", |_, _| async move { Response::ok("ok") })
        // Restaurants
        .get_async("/api/restaurants", |_, ctx| async move {
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            match repo.list_restaurants().await {
                Ok(restaurants) => json_ok(&restaurants, 200),
                Err(e) => repo_err_to_response(e),
            }
        })
        .get_async("/api/restaurants/:id", |_, ctx| async move {
            let id_str = ctx.param("id").unwrap().to_string();
            let id = RestaurantId::from_uuid(parse_uuid(&id_str));
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            match repo.get_restaurant(id).await {
                Ok(r) => json_ok(&r, 200),
                Err(e) => repo_err_to_response(e),
            }
        })
        .post_async("/api/restaurants", |mut req, ctx| async move {
            // Auth required
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };
            let body: CreateRestaurantReq = req.json().await?;
            let repo = D1Repo::new(ctx.env.d1("DB")?);

            let user = match require_active_user(&repo, &sub).await {
                Ok(u) => u,
                Err(resp) => return resp,
            };

            let result = repo
                .create_restaurant(CreateRestaurantRequest {
                    name: body.name,
                    zone_id: ZoneId::from_uuid(parse_uuid(&body.zone_id)),
                    menu: body
                        .menu
                        .into_iter()
                        .map(|m| CreateMenuItemRequest {
                            name: m.name,
                            price: Money::from(m.price.as_str()),
                        })
                        .collect(),
                    owner_id: Some(user.id),
                    description: None,
                    address: None,
                    phone: None,
                })
                .await;

            match result {
                Ok(r) => {
                    // Auto-promote to RestaurantOwner
                    if user.role == UserRole::Customer {
                        let _ = repo
                            .update_user_role(user.id, UserRole::RestaurantOwner)
                            .await;
                    }
                    json_ok(&r, 201)
                }
                Err(e) => repo_err_to_response(e),
            }
        })
        // Restaurant management (auth required)
        .patch_async("/api/restaurants/:id", |mut req, ctx| async move {
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };
            let id_str = ctx.param("id").unwrap().to_string();
            let id = RestaurantId::from_uuid(parse_uuid(&id_str));
            let body: UpdateRestaurantReq = req.json().await?;
            let repo = D1Repo::new(ctx.env.d1("DB")?);

            // Verify active + ownership
            let user = match require_active_user(&repo, &sub).await {
                Ok(u) => u,
                Err(resp) => return resp,
            };
            let restaurant = repo.get_restaurant(id).await.map_err(|e| Error::RustError(e.to_string()))?;
            match restaurant.owner_id {
                Some(oid) if oid == user.id => {}
                _ => return Response::error("not the owner", 403),
            }

            match repo.update_restaurant(id, UpdateRestaurantRequest {
                name: body.name,
                description: body.description,
                address: body.address,
                phone: body.phone,
            }).await {
                Ok(r) => json_ok(&r, 200),
                Err(e) => repo_err_to_response(e),
            }
        })
        .patch_async("/api/restaurants/:id/active", |mut req, ctx| async move {
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };
            let id_str = ctx.param("id").unwrap().to_string();
            let id = RestaurantId::from_uuid(parse_uuid(&id_str));
            let body: ToggleActiveReq = req.json().await?;
            let repo = D1Repo::new(ctx.env.d1("DB")?);

            let user = match require_active_user(&repo, &sub).await {
                Ok(u) => u,
                Err(resp) => return resp,
            };
            let restaurant = repo.get_restaurant(id).await.map_err(|e| Error::RustError(e.to_string()))?;
            match restaurant.owner_id {
                Some(oid) if oid == user.id => {}
                _ => return Response::error("not the owner", 403),
            }

            match repo.toggle_restaurant_active(id, body.active).await {
                Ok(r) => json_ok(&r, 200),
                Err(e) => repo_err_to_response(e),
            }
        })
        .post_async("/api/restaurants/:id/menu", |mut req, ctx| async move {
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };
            let id_str = ctx.param("id").unwrap().to_string();
            let id = RestaurantId::from_uuid(parse_uuid(&id_str));
            let body: CreateMenuItemReq = req.json().await?;
            let repo = D1Repo::new(ctx.env.d1("DB")?);

            let user = match require_active_user(&repo, &sub).await {
                Ok(u) => u,
                Err(resp) => return resp,
            };
            let restaurant = repo.get_restaurant(id).await.map_err(|e| Error::RustError(e.to_string()))?;
            match restaurant.owner_id {
                Some(oid) if oid == user.id => {}
                _ => return Response::error("not the owner", 403),
            }

            match repo.add_menu_item(id, CreateMenuItemRequest {
                name: body.name,
                price: Money::from(body.price.as_str()),
            }).await {
                Ok(item) => json_ok(&item, 201),
                Err(e) => repo_err_to_response(e),
            }
        })
        .patch_async("/api/menu-items/:id", |mut req, ctx| async move {
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };
            let id_str = ctx.param("id").unwrap().to_string();
            let id = MenuItemId::from_uuid(parse_uuid(&id_str));
            let body: UpdateMenuItemReq = req.json().await?;
            let repo = D1Repo::new(ctx.env.d1("DB")?);

            // Verify active + ownership BEFORE modifying
            let current = repo.get_menu_item(id).await.map_err(|e| Error::RustError(e.to_string()))?;
            let user = match require_active_user(&repo, &sub).await {
                Ok(u) => u,
                Err(resp) => return resp,
            };
            let restaurant = repo.get_restaurant(current.restaurant_id).await.map_err(|e| Error::RustError(e.to_string()))?;
            match restaurant.owner_id {
                Some(oid) if oid == user.id => {}
                _ => return Response::error("not the owner", 403),
            }

            let item = repo.update_menu_item(id, UpdateMenuItemRequest {
                name: body.name,
                price: body.price.map(|p| Money::from(p.as_str())),
            }).await.map_err(|e| Error::RustError(e.to_string()))?;

            json_ok(&item, 200)
        })
        .delete_async("/api/menu-items/:id", |req, ctx| async move {
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };
            let id_str = ctx.param("id").unwrap().to_string();
            let id = MenuItemId::from_uuid(parse_uuid(&id_str));
            let repo = D1Repo::new(ctx.env.d1("DB")?);

            // Verify active + ownership BEFORE deleting
            let item = repo.get_menu_item(id).await.map_err(|e| Error::RustError(e.to_string()))?;
            let user = match require_active_user(&repo, &sub).await {
                Ok(u) => u,
                Err(resp) => return resp,
            };
            let restaurant = repo.get_restaurant(item.restaurant_id).await.map_err(|e| Error::RustError(e.to_string()))?;
            match restaurant.owner_id {
                Some(oid) if oid == user.id => {}
                _ => return Response::error("not the owner", 403),
            }

            match repo.delete_menu_item(id).await {
                Ok(_) => Response::ok(""),
                Err(e) => repo_err_to_response(e),
            }
        })
        .get_async("/api/my/restaurants", |req, ctx| async move {
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            let user = match require_active_user(&repo, &sub).await {
                Ok(u) => u,
                Err(resp) => return resp,
            };
            match repo.list_restaurants_by_owner(user.id).await {
                Ok(restaurants) => json_ok(&restaurants, 200),
                Err(e) => repo_err_to_response(e),
            }
        })
        .get_async("/api/my/orders", |req, ctx| async move {
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            let user = match require_active_user(&repo, &sub).await {
                Ok(u) => u,
                Err(resp) => return resp,
            };
            let restaurants = match repo.list_restaurants_by_owner(user.id).await {
                Ok(r) => r,
                Err(e) => return repo_err_to_response(e),
            };
            let mut all_orders = Vec::new();
            for restaurant in &restaurants {
                match repo.list_restaurant_orders(restaurant.id).await {
                    Ok(orders) => all_orders.extend(orders),
                    Err(e) => return repo_err_to_response(e),
                }
            }
            all_orders.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            json_ok(&all_orders, 200)
        })
        // Orders
        .get_async("/api/orders", |_, ctx| async move {
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            match repo.list_orders().await {
                Ok(orders) => json_ok(&orders, 200),
                Err(e) => repo_err_to_response(e),
            }
        })
        .post_async("/api/orders", |mut req, ctx| async move {
            // Auth required + blocked check
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };

            let body: CreateOrderReq = req.json().await?;
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            if let Err(resp) = require_active_user(&repo, &sub).await {
                return resp;
            }
            let order = repo
                .create_order(CreateOrderRequest {
                    restaurant_id: RestaurantId::from_uuid(parse_uuid(&body.restaurant_id)),
                    items: body
                        .items
                        .into_iter()
                        .map(|i| CreateOrderItemRequest {
                            menu_item_id: MenuItemId::from_uuid(parse_uuid(&i.menu_item_id)),
                            name: i.name,
                            quantity: i.quantity,
                            unit_price: Money::from(i.unit_price.as_str()),
                        })
                        .collect(),
                    customer_address: body.customer_address,
                    zone_id: ZoneId::from_uuid(parse_uuid(&body.zone_id)),
                    delivery_fee: Money::from(body.delivery_fee.as_str()),
                    tip: Money::from(body.tip.as_str()),
                    local_ops_fee: Money::from(body.local_ops_fee.as_str()),
                })
                .await;

            match order {
                Ok(order) => {
                    let order_id_str = order.id.to_string();

                    // Create payment record
                    let payment_req = CreatePaymentRequest {
                        order_id: order.id,
                        stripe_checkout_session_id: None,
                        amount_total: order.pricing.total(),
                        restaurant_amount: order.pricing.food_total,
                        courier_amount: order.pricing.delivery_fee + order.pricing.tip,
                        federal_amount: order.pricing.federal_fee,
                        local_ops_amount: order.pricing.local_ops_fee,
                        processing_amount: order.pricing.processing_fee,
                    };

                    let payment = repo.create_payment(payment_req).await.ok();
                    let payment_id = payment.as_ref().map(|p| p.id.to_string());

                    // Try to create Stripe Checkout Session
                    let checkout_url = match ctx.env.secret("STRIPE_SECRET_KEY") {
                        Ok(key) => {
                            let stripe =
                                stripe_universal::StripeClient::new(key.to_string());
                            let origin = body
                                .origin_url
                                .unwrap_or_else(|| "https://openwok.superduperai.co".into());
                            let success_url =
                                format!("{origin}/order/{order_id_str}/success");
                            let cancel_url = format!("{origin}/checkout");

                            let total_cents = to_cents(&order.pricing.total());
                            let params =
                                stripe_universal::types::CreateCheckoutSessionParams {
                                    mode: stripe_universal::types::CheckoutMode::Payment,
                                    success_url,
                                    cancel_url,
                                    line_items: vec![stripe_universal::types::LineItem {
                                        price_data: stripe_universal::types::PriceData {
                                            currency: "usd".into(),
                                            product_data:
                                                stripe_universal::types::ProductData {
                                                    name: "OpenWok Order".into(),
                                                },
                                            unit_amount: total_cents,
                                        },
                                        quantity: 1,
                                    }],
                                    payment_intent_data: None,
                                    metadata: Some(
                                        [(
                                            "order_id".to_string(),
                                            order_id_str.clone(),
                                        )]
                                        .into_iter()
                                        .collect(),
                                    ),
                                };

                            match stripe.create_checkout_session(&params).await {
                                Ok(session) => session.url,
                                Err(_) => None,
                            }
                        }
                        Err(_) => None, // No Stripe key — dev mode
                    };

                    json_ok(
                        &serde_json::json!({
                            "order": order,
                            "checkout_url": checkout_url,
                            "payment_id": payment_id,
                        }),
                        201,
                    )
                }
                Err(e) => repo_err_to_response(e),
            }
        })
        .get_async("/api/orders/:id", |_, ctx| async move {
            let id_str = ctx.param("id").unwrap().to_string();
            let id = OrderId::from_uuid(parse_uuid(&id_str));
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            match repo.get_order(id).await {
                Ok(order) => json_ok(&order, 200),
                Err(e) => repo_err_to_response(e),
            }
        })
        .patch_async("/api/orders/:id/status", |mut req, ctx| async move {
            // Auth required + blocked check
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };

            let id_str = ctx.param("id").unwrap().to_string();
            let body: TransitionReq = req.json().await?;
            let id = OrderId::from_uuid(parse_uuid(&id_str));
            let status = parse_status(&body.status)
                .ok_or_else(|| Error::RustError("invalid status".into()))?;
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            if let Err(resp) = require_active_user(&repo, &sub).await {
                return resp;
            }
            match repo.update_order_status(id, status).await {
                Ok(order) => json_ok(&order, 200),
                Err(e) => repo_err_to_response(e),
            }
        })
        .post_async("/api/orders/:id/assign", |req, ctx| async move {
            // Auth required + blocked check
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };

            let id_str = ctx.param("id").unwrap().to_string();
            let id = OrderId::from_uuid(parse_uuid(&id_str));
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            if let Err(resp) = require_active_user(&repo, &sub).await {
                return resp;
            }
            match repo.assign_courier(id).await {
                Ok(result) => json_ok(
                    &serde_json::json!({"order_id": result.order_id, "courier_id": result.courier_id}),
                    200,
                ),
                Err(e) => repo_err_to_response(e),
            }
        })
        // Couriers
        .get_async("/api/couriers", |_, ctx| async move {
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            match repo.list_couriers().await {
                Ok(couriers) => json_ok(&couriers, 200),
                Err(e) => repo_err_to_response(e),
            }
        })
        .post_async("/api/couriers", |mut req, ctx| async move {
            // Auth required + blocked check
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };

            let body: CreateCourierReq = req.json().await?;
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            if let Err(resp) = require_active_user(&repo, &sub).await {
                return resp;
            }
            let result = repo
                .create_courier(CreateCourierRequest {
                    name: body.name,
                    zone_id: ZoneId::from_uuid(parse_uuid(&body.zone_id)),
                    user_id: None,
                })
                .await;
            match result {
                Ok(courier) => json_ok(&courier, 201),
                Err(e) => repo_err_to_response(e),
            }
        })
        .patch_async("/api/couriers/:id/available", |mut req, ctx| async move {
            // Auth required + blocked check
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };

            let repo_check = D1Repo::new(ctx.env.d1("DB")?);
            if let Err(resp) = require_active_user(&repo_check, &sub).await {
                return resp;
            }

            let id_str = ctx.param("id").unwrap().to_string();
            let body: SetAvailableReq = req.json().await?;
            let id = CourierId::from_uuid(parse_uuid(&id_str));
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            match repo.toggle_courier_available(id, body.available).await {
                Ok(courier) => json_ok(&courier, 200),
                Err(e) => repo_err_to_response(e),
            }
        })
        // Economics
        .get_async("/api/public/economics", |_, ctx| async move {
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            match repo.get_economics().await {
                Ok(economics) => {
                    let mut resp = json_ok(&economics, 200)?;
                    resp.headers_mut()
                        .set("Cache-Control", "public, max-age=300")?;
                    Ok(resp)
                }
                Err(e) => repo_err_to_response(e),
            }
        })
        // Metrics
        .get_async("/api/admin/metrics", |_, ctx| async move {
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            match repo.get_metrics().await {
                Ok(metrics) => json_ok(&metrics, 200),
                Err(e) => repo_err_to_response(e),
            }
        })
        // Auth
        .post_async("/api/auth/callback", |mut req, ctx| async move {
            let body: AuthCallbackReq = req.json().await?;
            let secret = ctx.env
                .secret("SUPABASE_JWT_SECRET")
                .map(|s| s.to_string())
                .unwrap_or_else(|_| "super-secret-jwt-token-for-testing-only".into());

            let (sub, email) = verify_jwt(&body.access_token, &secret)?;

            let repo = D1Repo::new(ctx.env.d1("DB")?);
            let user = match repo.get_user_by_supabase_id(&sub).await {
                Ok(user) => user,
                Err(_) => {
                    repo.create_user(CreateUserRequest {
                        supabase_user_id: sub,
                        email: email.unwrap_or_default(),
                        name: None,
                        role: None,
                    })
                    .await
                    .map_err(|e| Error::RustError(e.to_string()))?
                }
            };

            json_ok(
                &serde_json::json!({ "user": user, "access_token": body.access_token }),
                200,
            )
        })
        .get_async("/api/auth/me", |req, ctx| async move {
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            match repo.get_user_by_supabase_id(&sub).await {
                Ok(user) => json_ok(&user, 200),
                Err(_) => Response::error("user not found", 404),
            }
        })
        // Stripe webhook
        .post_async("/api/webhooks/stripe", |mut req, ctx| async move {
            let webhook_secret = ctx
                .env
                .secret("STRIPE_WEBHOOK_SECRET")
                .map(|s| s.to_string())
                .map_err(|_| Error::RustError("webhook secret not configured".into()))?;

            let signature = req
                .headers()
                .get("stripe-signature")
                .map_err(|_| Error::RustError("missing header".into()))?
                .ok_or_else(|| Error::RustError("missing stripe-signature".into()))?;

            let body_bytes = req.bytes().await?;

            stripe_universal::webhook::verify_and_parse(&body_bytes, &signature, &webhook_secret)
                .map_err(|e| Error::RustError(format!("webhook verify failed: {e}")))?;

            let body_str = String::from_utf8_lossy(&body_bytes);
            let event: stripe_universal::types::WebhookEvent = serde_json::from_str(&body_str)
                .map_err(|e| Error::RustError(format!("invalid event: {e}")))?;

            let repo = D1Repo::new(ctx.env.d1("DB")?);

            match event.event_type.as_str() {
                "checkout.session.completed" => {
                    let session = event
                        .as_checkout_session()
                        .map_err(|e| Error::RustError(e.to_string()))?;
                    if let Some(order_id_str) = session.metadata.as_ref().and_then(|m| m.get("order_id")) {
                        let order_id = OrderId::from_uuid(parse_uuid(order_id_str));
                        if let Ok(payment) = repo.get_payment_by_order(order_id).await {
                            let _ = repo
                                .update_payment_status(
                                    payment.id,
                                    UpdatePaymentStatusRequest {
                                        status: PaymentStatus::Succeeded,
                                        stripe_payment_intent_id: session.payment_intent,
                                    },
                                )
                                .await;
                            let _ = repo.update_order_status(order_id, OrderStatus::Confirmed).await;
                        }
                    }
                }
                "checkout.session.expired" => {
                    let session = event
                        .as_checkout_session()
                        .map_err(|e| Error::RustError(e.to_string()))?;
                    if let Some(order_id_str) = session.metadata.as_ref().and_then(|m| m.get("order_id")) {
                        let order_id = OrderId::from_uuid(parse_uuid(order_id_str));
                        if let Ok(payment) = repo.get_payment_by_order(order_id).await {
                            let _ = repo
                                .update_payment_status(
                                    payment.id,
                                    UpdatePaymentStatusRequest {
                                        status: PaymentStatus::Failed,
                                        stripe_payment_intent_id: None,
                                    },
                                )
                                .await;
                            let _ = repo.update_order_status(order_id, OrderStatus::Cancelled).await;
                        }
                    }
                }
                _ => {}
            }

            Response::ok("ok")
        })
        // Admin: users
        .get_async("/api/admin/users", |req, ctx| async move {
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            let user = repo.get_user_by_supabase_id(&sub).await.map_err(|_| Error::RustError("user not found".into()))?;
            if user.role != UserRole::NodeOperator || user.blocked {
                return Response::error("admin access required", 403);
            }
            match repo.list_users().await {
                Ok(users) => json_ok(&users, 200),
                Err(e) => repo_err_to_response(e),
            }
        })
        .patch_async("/api/admin/users/:id/block", |mut req, ctx| async move {
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            let user = repo.get_user_by_supabase_id(&sub).await.map_err(|_| Error::RustError("user not found".into()))?;
            if user.role != UserRole::NodeOperator || user.blocked {
                return Response::error("admin access required", 403);
            }
            let id_str = ctx.param("id").unwrap().to_string();
            let target_id = openwok_core::types::UserId::from_uuid(parse_uuid(&id_str));
            #[derive(Deserialize)]
            struct BlockReq { blocked: bool }
            let body: BlockReq = req.json().await?;
            match repo.set_user_blocked(target_id, body.blocked).await {
                Ok(u) => json_ok(&u, 200),
                Err(e) => repo_err_to_response(e),
            }
        })
        // Admin: disputes
        .get_async("/api/admin/disputes", |req, ctx| async move {
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            let user = repo.get_user_by_supabase_id(&sub).await.map_err(|_| Error::RustError("user not found".into()))?;
            if user.role != UserRole::NodeOperator || user.blocked {
                return Response::error("admin access required", 403);
            }
            match repo.list_disputes().await {
                Ok(disputes) => json_ok(&disputes, 200),
                Err(e) => repo_err_to_response(e),
            }
        })
        .patch_async("/api/admin/disputes/:id/resolve", |mut req, ctx| async move {
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            let user = repo.get_user_by_supabase_id(&sub).await.map_err(|_| Error::RustError("user not found".into()))?;
            if user.role != UserRole::NodeOperator || user.blocked {
                return Response::error("admin access required", 403);
            }
            let id_str = ctx.param("id").unwrap().to_string();
            let dispute_id = openwok_core::types::DisputeId::from_uuid(parse_uuid(&id_str));
            #[derive(Deserialize)]
            struct ResolveReq { status: String, resolution: Option<String> }
            let body: ResolveReq = req.json().await?;
            let status = match body.status.as_str() {
                "Resolved" => openwok_core::types::DisputeStatus::Resolved,
                "Dismissed" => openwok_core::types::DisputeStatus::Dismissed,
                _ => return Response::error("invalid status", 400),
            };
            match repo.resolve_dispute(dispute_id, status, body.resolution).await {
                Ok(d) => json_ok(&d, 200),
                Err(e) => repo_err_to_response(e),
            }
        })
        // Dispute creation (any auth user)
        .post_async("/api/orders/:id/dispute", |mut req, ctx| async move {
            let (sub, _) = match extract_auth(&req, &ctx.env) {
                Ok(v) => v,
                Err(e) => return auth_err_to_response(e),
            };
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            let user = repo.get_user_by_supabase_id(&sub).await.map_err(|_| Error::RustError("user not found".into()))?;
            if user.blocked {
                return Response::error("user is blocked", 403);
            }
            let id_str = ctx.param("id").unwrap().to_string();
            let order_id = OrderId::from_uuid(parse_uuid(&id_str));
            #[derive(Deserialize)]
            struct DisputeReq { reason: String }
            let body: DisputeReq = req.json().await?;
            match repo.create_dispute(order_id, user.id, body.reason).await {
                Ok(d) => json_ok(&d, 201),
                Err(e) => repo_err_to_response(e),
            }
        })
        .run(req, env)
        .await
}
