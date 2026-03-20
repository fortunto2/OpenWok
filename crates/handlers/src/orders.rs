use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use openwok_core::money::Money;
use openwok_core::order::{Order, OrderStatus};
use openwok_core::repo::{CreateOrderItemRequest, CreateOrderRequest, Repository};

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
    State(repo): State<Arc<R>>,
    Json(body): Json<CreateOrder>,
) -> Result<(StatusCode, Json<Order>), (StatusCode, String)> {
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
    repo.create_order(req)
        .await
        .map(|order| (StatusCode::CREATED, Json(order)))
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))
}

#[utoipa::path(get, path = "/orders/{id}", tag = "orders")]
pub async fn get<R: Repository>(
    State(repo): State<Arc<R>>,
    Path(id): Path<OrderId>,
) -> Result<Json<Order>, StatusCode> {
    repo.get_order(id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

#[utoipa::path(patch, path = "/orders/{id}/status", tag = "orders")]
pub async fn transition<R: Repository>(
    _auth: AuthUser,
    State(repo): State<Arc<R>>,
    Path(id): Path<OrderId>,
    Json(body): Json<TransitionStatus>,
) -> Result<Json<Order>, (StatusCode, String)> {
    repo.update_order_status(id, body.status)
        .await
        .map(Json)
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))
}
