use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use openwok_core::types::{Courier, CourierId, CourierKind, OrderId, ZoneId};
use rusqlite::params;
use serde::Deserialize;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateCourier {
    pub name: String,
    pub zone_id: ZoneId,
}

#[derive(Deserialize)]
pub struct SetAvailable {
    pub available: bool,
}

fn row_to_courier(id: &str, name: String, kind: &str, zone_id: &str, available: bool) -> Courier {
    Courier {
        id: CourierId::from_uuid(uuid::Uuid::parse_str(id).unwrap()),
        name,
        kind: match kind {
            "Human" => CourierKind::Human,
            _ => CourierKind::Human,
        },
        zone_id: ZoneId::from_uuid(uuid::Uuid::parse_str(zone_id).unwrap()),
        available,
    }
}

pub async fn list(State(state): State<AppState>) -> Json<Vec<Courier>> {
    let conn = state.db.lock().await;
    let mut stmt = conn
        .prepare("SELECT id, name, kind, zone_id, available FROM couriers WHERE available = 1")
        .unwrap();
    let couriers: Vec<Courier> = stmt
        .query_map([], |row| {
            Ok(row_to_courier(
                &row.get::<_, String>(0)?,
                row.get(1)?,
                &row.get::<_, String>(2)?,
                &row.get::<_, String>(3)?,
                row.get(4)?,
            ))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    Json(couriers)
}

pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateCourier>,
) -> (StatusCode, Json<Courier>) {
    let conn = state.db.lock().await;
    let id = CourierId::new();
    let id_str = id.to_string();
    let zone_str = body.zone_id.to_string();

    // Ensure zone exists
    conn.execute(
        "INSERT OR IGNORE INTO zones (id, name) VALUES (?1, ?2)",
        params![zone_str, format!("Zone {}", &zone_str[..8])],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO couriers (id, name, kind, zone_id, available) VALUES (?1, ?2, ?3, ?4, 1)",
        params![id_str, body.name, "Human", zone_str],
    )
    .unwrap();

    let courier = Courier {
        id,
        name: body.name,
        kind: CourierKind::Human,
        zone_id: body.zone_id,
        available: true,
    };

    (StatusCode::CREATED, Json(courier))
}

pub async fn toggle_available(
    State(state): State<AppState>,
    Path(id): Path<CourierId>,
    Json(body): Json<SetAvailable>,
) -> Result<Json<Courier>, StatusCode> {
    let conn = state.db.lock().await;
    let id_str = id.to_string();

    let updated = conn
        .execute(
            "UPDATE couriers SET available = ?1 WHERE id = ?2",
            params![body.available, id_str],
        )
        .unwrap();

    if updated == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    let courier = conn
        .query_row(
            "SELECT id, name, kind, zone_id, available FROM couriers WHERE id = ?1",
            params![id_str],
            |row| {
                Ok(row_to_courier(
                    &row.get::<_, String>(0)?,
                    row.get(1)?,
                    &row.get::<_, String>(2)?,
                    &row.get::<_, String>(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(courier))
}

pub async fn assign_to_order(
    State(state): State<AppState>,
    Path(order_id): Path<OrderId>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let conn = state.db.lock().await;
    let order_id_str = order_id.to_string();

    // Get order's zone
    let zone_id: String = conn
        .query_row(
            "SELECT zone_id FROM orders WHERE id = ?1",
            params![order_id_str],
            |r| r.get(0),
        )
        .map_err(|_| (StatusCode::NOT_FOUND, "order not found".into()))?;

    // Find available courier in zone
    let courier_id: String = conn
        .query_row(
            "SELECT id FROM couriers WHERE available = 1 AND zone_id = ?1 LIMIT 1",
            params![zone_id],
            |r| r.get(0),
        )
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "no available courier in zone".into(),
            )
        })?;

    // Mark courier unavailable
    conn.execute(
        "UPDATE couriers SET available = 0 WHERE id = ?1",
        params![courier_id],
    )
    .unwrap();

    // Assign courier to order
    conn.execute(
        "UPDATE orders SET courier_id = ?1 WHERE id = ?2",
        params![courier_id, order_id_str],
    )
    .unwrap();

    Ok(Json(serde_json::json!({
        "order_id": order_id_str,
        "courier_id": courier_id,
    })))
}
