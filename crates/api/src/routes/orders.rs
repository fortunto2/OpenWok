use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use openwok_core::money::Money;
use openwok_core::order::{Order, OrderItem, OrderStatus};
use openwok_core::types::{MenuItemId, OrderId, RestaurantId, ZoneId};
use serde::Deserialize;

use crate::state::SharedState;

#[derive(Deserialize)]
pub struct CreateOrder {
    pub restaurant_id: RestaurantId,
    pub items: Vec<CreateOrderItem>,
    pub customer_address: String,
    pub zone_id: ZoneId,
    pub delivery_fee: Money,
    pub tip: Money,
    pub local_ops_fee: Money,
}

#[derive(Deserialize)]
pub struct CreateOrderItem {
    pub menu_item_id: MenuItemId,
    pub name: String,
    pub quantity: u32,
    pub unit_price: Money,
}

#[derive(Deserialize)]
pub struct TransitionStatus {
    pub status: OrderStatus,
}

pub async fn create(
    State(state): State<SharedState>,
    Json(body): Json<CreateOrder>,
) -> Result<(StatusCode, Json<Order>), (StatusCode, String)> {
    // Validate restaurant exists
    {
        let s = state.read().await;
        if !s.restaurants.contains_key(&body.restaurant_id) {
            return Err((
                StatusCode::NOT_FOUND,
                "restaurant not found".into(),
            ));
        }
    }

    let items: Vec<OrderItem> = body
        .items
        .into_iter()
        .map(|i| OrderItem {
            menu_item_id: i.menu_item_id,
            name: i.name,
            quantity: i.quantity,
            unit_price: i.unit_price,
        })
        .collect();

    let order = Order::new(
        items,
        body.restaurant_id,
        body.customer_address,
        body.zone_id,
        body.delivery_fee,
        body.tip,
        body.local_ops_fee,
    )
    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let mut s = state.write().await;
    s.orders.insert(order.id, order.clone());
    Ok((StatusCode::CREATED, Json(order)))
}

pub async fn get(
    State(state): State<SharedState>,
    Path(id): Path<OrderId>,
) -> Result<Json<Order>, StatusCode> {
    let s = state.read().await;
    s.orders.get(&id).cloned().map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub async fn transition(
    State(state): State<SharedState>,
    Path(id): Path<OrderId>,
    Json(body): Json<TransitionStatus>,
) -> Result<Json<Order>, (StatusCode, String)> {
    let mut s = state.write().await;
    let order = s
        .orders
        .get_mut(&id)
        .ok_or((StatusCode::NOT_FOUND, "order not found".into()))?;

    order
        .transition(body.status)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(order.clone()))
}
