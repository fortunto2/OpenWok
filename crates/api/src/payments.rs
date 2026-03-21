use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use openwok_core::application::{orders as order_app, payments as payment_app};
use openwok_core::money::Money;
use serde::{Deserialize, Serialize};

use openwok_handlers::auth::AuthUser;

use crate::state::AppState;
use crate::stripe::build_checkout_params;

#[derive(Deserialize)]
pub struct CreateOrderWithPayment {
    pub restaurant_id: openwok_core::types::RestaurantId,
    pub items: Vec<OrderItemInput>,
    pub customer_address: String,
    pub zone_id: openwok_core::types::ZoneId,
    pub delivery_fee: Money,
    pub tip: Money,
    pub local_ops_fee: Money,
    /// Frontend origin for success/cancel redirect URLs
    pub origin_url: Option<String>,
}

#[derive(Deserialize)]
pub struct OrderItemInput {
    pub menu_item_id: openwok_core::types::MenuItemId,
    pub name: String,
    pub quantity: u32,
    pub unit_price: Money,
}

#[derive(Serialize)]
pub struct OrderWithCheckout {
    pub order: openwok_core::order::Order,
    pub checkout_url: Option<String>,
    pub payment_id: Option<String>,
}

/// POST /api/orders — creates order + payment record, returns Stripe checkout URL.
pub async fn create_order_with_payment(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateOrderWithPayment>,
) -> Result<(StatusCode, Json<OrderWithCheckout>), (StatusCode, String)> {
    let repo = &state.repo;

    let order_req = openwok_core::repo::CreateOrderRequest {
        restaurant_id: body.restaurant_id,
        items: body
            .items
            .into_iter()
            .map(|i| openwok_core::repo::CreateOrderItemRequest {
                menu_item_id: i.menu_item_id,
                name: i.name,
                quantity: i.quantity,
                unit_price: i.unit_price,
            })
            .collect(),
        customer_address: body.customer_address,
        zone_id: body.zone_id,
        delivery_fee: body.delivery_fee,
        tip: body.tip,
        local_ops_fee: body.local_ops_fee,
    };

    let order = order_app::create_order(repo.as_ref(), order_req)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let order_id_str = order.id.to_string();

    let payment = payment_app::create_payment_for_order(repo.as_ref(), &order)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Try to create Stripe Checkout Session
    let checkout_url = if let Some(ref stripe) = state.stripe_client {
        let allowed_origins = [
            "https://openwok.superduperai.co",
            "http://localhost:8080",
            "http://localhost:3000",
            "http://localhost:3030",
        ];
        let origin = body
            .origin_url
            .filter(|u| allowed_origins.iter().any(|a| u.starts_with(a)))
            .unwrap_or_else(|| "https://openwok.superduperai.co".into());
        let success_url = format!("{origin}/order/{order_id_str}/success");
        let cancel_url = format!("{origin}/checkout");

        let params = build_checkout_params(
            &order.pricing,
            &order_id_str,
            &success_url,
            &cancel_url,
            None, // No Connect accounts in MVP
        );

        match stripe.create_checkout_session(&params).await {
            Ok(session) => session.url,
            Err(e) => {
                eprintln!("Stripe checkout creation failed: {e}");
                None
            }
        }
    } else {
        // No Stripe key configured — development mode
        None
    };

    Ok((
        StatusCode::CREATED,
        Json(OrderWithCheckout {
            order,
            checkout_url,
            payment_id: Some(payment.id.to_string()),
        }),
    ))
}

/// POST /api/webhooks/stripe — handles Stripe webhook events.
pub async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, (StatusCode, String)> {
    let webhook_secret = state.stripe_webhook_secret.as_deref().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "webhook secret not configured".into(),
    ))?;

    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::BAD_REQUEST,
            "missing stripe-signature header".into(),
        ))?;

    let body_str = std::str::from_utf8(&body)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid body encoding".into()))?;

    // Verify webhook signature
    stripe_universal::webhook::verify_and_parse(body_str.as_bytes(), signature, webhook_secret)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("webhook verification failed: {e}"),
            )
        })?;

    // Parse the event
    let event: stripe_universal::types::WebhookEvent = serde_json::from_str(body_str)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("invalid event: {e}")))?;

    match event.event_type.as_str() {
        "checkout.session.completed" => {
            let session = event.as_checkout_session().map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("invalid session data: {e}"),
                )
            })?;

            if let Some(order_id_str) = session.metadata.as_ref().and_then(|m| m.get("order_id")) {
                let order_id = uuid::Uuid::parse_str(order_id_str)
                    .map(openwok_core::types::OrderId::from_uuid)
                    .map_err(|_| {
                        (
                            StatusCode::BAD_REQUEST,
                            "invalid order_id in metadata".into(),
                        )
                    })?;

                payment_app::mark_payment_succeeded(
                    state.repo.as_ref(),
                    order_id,
                    session.payment_intent,
                )
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            }
        }
        "checkout.session.expired" => {
            let session = event.as_checkout_session().map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("invalid session data: {e}"),
                )
            })?;

            if let Some(order_id_str) = session.metadata.as_ref().and_then(|m| m.get("order_id")) {
                let order_id = uuid::Uuid::parse_str(order_id_str)
                    .map(openwok_core::types::OrderId::from_uuid)
                    .map_err(|_| (StatusCode::BAD_REQUEST, "invalid order_id".into()))?;

                payment_app::mark_payment_failed(state.repo.as_ref(), order_id)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            }
        }
        _ => {
            // Ignore unknown event types
        }
    }

    Ok(StatusCode::OK)
}
