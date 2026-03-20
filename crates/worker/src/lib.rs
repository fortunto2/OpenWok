mod d1_repo;

use d1_repo::D1Repo;
use openwok_core::money::Money;
use openwok_core::order::OrderStatus;
use openwok_core::repo::{
    CreateCourierRequest, CreateMenuItemRequest, CreateOrderItemRequest, CreateOrderRequest,
    CreateRestaurantRequest,
};
use openwok_core::types::{
    CourierId, MenuItemId, OrderId, RestaurantId, ZoneId,
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
            let body: CreateRestaurantReq = req.json().await?;
            let repo = D1Repo::new(ctx.env.d1("DB")?);
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
                })
                .await;
            match result {
                Ok(r) => json_ok(&r, 201),
                Err(e) => repo_err_to_response(e),
            }
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
            let body: CreateOrderReq = req.json().await?;
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            let result = repo
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
            match result {
                Ok(order) => json_ok(&order, 201),
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
            let id_str = ctx.param("id").unwrap().to_string();
            let body: TransitionReq = req.json().await?;
            let id = OrderId::from_uuid(parse_uuid(&id_str));
            let status = parse_status(&body.status)
                .ok_or_else(|| Error::RustError("invalid status".into()))?;
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            match repo.update_order_status(id, status).await {
                Ok(order) => json_ok(&order, 200),
                Err(e) => repo_err_to_response(e),
            }
        })
        .post_async("/api/orders/:id/assign", |_, ctx| async move {
            let id_str = ctx.param("id").unwrap().to_string();
            let id = OrderId::from_uuid(parse_uuid(&id_str));
            let repo = D1Repo::new(ctx.env.d1("DB")?);
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
            let body: CreateCourierReq = req.json().await?;
            let repo = D1Repo::new(ctx.env.d1("DB")?);
            let result = repo
                .create_courier(CreateCourierRequest {
                    name: body.name,
                    zone_id: ZoneId::from_uuid(parse_uuid(&body.zone_id)),
                })
                .await;
            match result {
                Ok(courier) => json_ok(&courier, 201),
                Err(e) => repo_err_to_response(e),
            }
        })
        .patch_async("/api/couriers/:id/available", |mut req, ctx| async move {
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
        .run(req, env)
        .await
}
