use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use openwok_core::application::orders as order_app;
use openwok_core::dispatch::OrderEvent;
use openwok_core::money::Money;
use openwok_core::order::{Order, OrderStatus};
use openwok_core::repo::{CreateOrderItemRequest, CreateOrderRequest, Repository};
use tokio::sync::broadcast;

use crate::admin::get_active_user;
use crate::auth::AuthUser;
use crate::restaurants::repo_error_to_status;
use openwok_core::types::{MenuItemId, OrderId, RestaurantId, ZoneId};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateOrder {
    pub restaurant_id: RestaurantId,
    pub items: Vec<CreateOrderItem>,
    pub customer_address: String,
    pub zone_id: ZoneId,
    pub delivery_fee: Money,
    pub tip: Money,
    pub local_ops_fee: Money,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateOrderItem {
    pub menu_item_id: MenuItemId,
    pub name: String,
    pub quantity: u32,
    pub unit_price: Money,
}

#[derive(Deserialize, ToSchema)]
pub struct TransitionStatus {
    pub status: OrderStatus,
}

#[utoipa::path(get, path = "/orders", tag = "orders")]
pub async fn list<R: Repository>(State(repo): State<Arc<R>>) -> Json<Vec<Order>> {
    Json(repo.list_orders().await.unwrap_or_default())
}

#[utoipa::path(post, path = "/orders", tag = "orders")]
pub async fn create<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
    Json(body): Json<CreateOrder>,
) -> Result<(StatusCode, Json<Order>), (StatusCode, String)> {
    get_active_user(repo.as_ref(), &auth).await?;
    let req = CreateOrderRequest {
        restaurant_id: body.restaurant_id,
        items: body
            .items
            .into_iter()
            .map(|i| CreateOrderItemRequest {
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
    order_app::create_order(repo.as_ref(), req)
        .await
        .map(|order| (StatusCode::CREATED, Json(order)))
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))
}

#[utoipa::path(get, path = "/orders/{id}", tag = "orders")]
pub async fn get<R: Repository>(
    State(repo): State<Arc<R>>,
    Path(id): Path<OrderId>,
) -> Result<Json<Order>, StatusCode> {
    order_app::get_order(repo.as_ref(), id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

#[utoipa::path(patch, path = "/orders/{id}/status", tag = "orders")]
pub async fn transition<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
    tx: Option<axum::Extension<broadcast::Sender<OrderEvent>>>,
    Path(id): Path<OrderId>,
    Json(body): Json<TransitionStatus>,
) -> Result<Json<Order>, (StatusCode, String)> {
    get_active_user(repo.as_ref(), &auth).await?;
    let result = order_app::transition_order(repo.as_ref(), id, body.status)
        .await
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))?;

    let sender = tx.map(|axum::Extension(s)| s);
    if let Some(ref s) = sender {
        for event in &result.events {
            let _ = s.send(OrderEvent {
                order_id: event.order_id.clone(),
                status: event.status.clone(),
            });
        }
    }

    Ok(Json(result.order))
}

/// POST /orders/{id}/dispute — any authenticated user can dispute their order
#[utoipa::path(post, path = "/orders/{id}/dispute", tag = "orders")]
pub async fn create_dispute<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
    Path(id): Path<OrderId>,
    Json(body): Json<openwok_core::types::CreateDisputeRequest>,
) -> Result<(StatusCode, Json<openwok_core::types::Dispute>), (StatusCode, String)> {
    let user = crate::admin::get_active_user(repo.as_ref(), &auth).await?;
    repo.create_dispute(id, user.id, body.reason)
        .await
        .map(|d| (StatusCode::CREATED, Json(d)))
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))
}
