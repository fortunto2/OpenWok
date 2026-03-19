use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use openwok_core::types::{Courier, CourierId, CourierKind, OrderId, ZoneId};
use serde::Deserialize;

use crate::state::SharedState;

#[derive(Deserialize)]
pub struct CreateCourier {
    pub name: String,
    pub zone_id: ZoneId,
}

#[derive(Deserialize)]
pub struct SetAvailable {
    pub available: bool,
}

pub async fn list(State(state): State<SharedState>) -> Json<Vec<Courier>> {
    let s = state.read().await;
    Json(
        s.couriers
            .values()
            .filter(|c| c.available)
            .cloned()
            .collect(),
    )
}

pub async fn create(
    State(state): State<SharedState>,
    Json(body): Json<CreateCourier>,
) -> (StatusCode, Json<Courier>) {
    let id = CourierId::new();
    let courier = Courier {
        id,
        name: body.name,
        kind: CourierKind::Human,
        zone_id: body.zone_id,
        available: true,
    };

    let mut s = state.write().await;
    s.couriers.insert(id, courier.clone());
    (StatusCode::CREATED, Json(courier))
}

pub async fn toggle_available(
    State(state): State<SharedState>,
    Path(id): Path<CourierId>,
    Json(body): Json<SetAvailable>,
) -> Result<Json<Courier>, StatusCode> {
    let mut s = state.write().await;
    let courier = s.couriers.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    courier.available = body.available;
    Ok(Json(courier.clone()))
}

pub async fn assign_to_order(
    State(state): State<SharedState>,
    Path(order_id): Path<OrderId>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state.write().await;

    // Find an available courier in the order's zone
    let order = s
        .orders
        .get(&order_id)
        .ok_or((StatusCode::NOT_FOUND, "order not found".into()))?;
    let zone = order.zone_id;

    let courier_id = s
        .couriers
        .values()
        .find(|c| c.available && c.zone_id == zone)
        .map(|c| c.id)
        .ok_or((
            StatusCode::BAD_REQUEST,
            "no available courier in zone".into(),
        ))?;

    // Mark courier unavailable
    if let Some(c) = s.couriers.get_mut(&courier_id) {
        c.available = false;
    }

    // Assign to order
    let order = s
        .orders
        .get_mut(&order_id)
        .ok_or((StatusCode::NOT_FOUND, "order not found".into()))?;
    order.courier_id = Some(courier_id);

    Ok(Json(serde_json::json!({
        "order_id": order_id,
        "courier_id": courier_id,
    })))
}
