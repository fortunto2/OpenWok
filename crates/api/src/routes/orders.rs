use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::Utc;
use openwok_core::money::Money;
use openwok_core::order::{Order, OrderItem, OrderStatus};
use openwok_core::pricing::PricingBreakdown;
use openwok_core::types::{CourierId, MenuItemId, OrderId, RestaurantId, ZoneId};
use rusqlite::params;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::state::{AppState, OrderEvent};

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

fn load_order(conn: &rusqlite::Connection, order_id: &str) -> Option<Order> {
    let row = conn.query_row(
        "SELECT id, restaurant_id, courier_id, customer_address, zone_id, status,
                food_total, delivery_fee, tip, federal_fee, local_ops_fee, processing_fee,
                created_at, updated_at, estimated_eta, actual_delivery_at
         FROM orders WHERE id = ?1",
        params![order_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, String>(11)?,
                row.get::<_, String>(12)?,
                row.get::<_, String>(13)?,
                row.get::<_, Option<i32>>(14)?,
                row.get::<_, Option<String>>(15)?,
            ))
        },
    );

    let (
        id,
        rest_id,
        courier_id,
        addr,
        zone_id,
        status,
        food_total,
        delivery_fee,
        tip,
        federal_fee,
        local_ops_fee,
        processing_fee,
        created_at,
        updated_at,
        estimated_eta,
        actual_delivery_at,
    ) = row.ok()?;

    let mut stmt = conn
        .prepare(
            "SELECT menu_item_id, name, quantity, unit_price FROM order_items WHERE order_id = ?1",
        )
        .unwrap();
    let items: Vec<OrderItem> = stmt
        .query_map(params![id], |r| {
            Ok(OrderItem {
                menu_item_id: MenuItemId::from_uuid(
                    uuid::Uuid::parse_str(&r.get::<_, String>(0)?).unwrap(),
                ),
                name: r.get(1)?,
                quantity: r.get::<_, u32>(2)?,
                unit_price: Money::from(r.get::<_, String>(3)?.as_str()),
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let status = match status.as_str() {
        "Created" => OrderStatus::Created,
        "Confirmed" => OrderStatus::Confirmed,
        "Preparing" => OrderStatus::Preparing,
        "ReadyForPickup" => OrderStatus::ReadyForPickup,
        "InDelivery" => OrderStatus::InDelivery,
        "Delivered" => OrderStatus::Delivered,
        "Cancelled" => OrderStatus::Cancelled,
        _ => return None,
    };

    Some(Order {
        id: OrderId::from_uuid(uuid::Uuid::parse_str(&id).unwrap()),
        items,
        restaurant_id: RestaurantId::from_uuid(uuid::Uuid::parse_str(&rest_id).unwrap()),
        courier_id: courier_id.map(|c| CourierId::from_uuid(uuid::Uuid::parse_str(&c).unwrap())),
        customer_address: addr,
        zone_id: ZoneId::from_uuid(uuid::Uuid::parse_str(&zone_id).unwrap()),
        status,
        pricing: PricingBreakdown {
            food_total: Money::from(food_total.as_str()),
            delivery_fee: Money::from(delivery_fee.as_str()),
            tip: Money::from(tip.as_str()),
            federal_fee: Money::from(federal_fee.as_str()),
            local_ops_fee: Money::from(local_ops_fee.as_str()),
            processing_fee: Money::from(processing_fee.as_str()),
        },
        created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
            .unwrap()
            .with_timezone(&Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
            .unwrap()
            .with_timezone(&Utc),
        estimated_eta,
        actual_delivery_at: actual_delivery_at.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        }),
    })
}

#[utoipa::path(get, path = "/orders", tag = "orders")]
pub async fn list(State(state): State<AppState>) -> Json<Vec<Order>> {
    let conn = state.db.lock().await;
    let mut stmt = conn.prepare("SELECT id FROM orders").unwrap();
    let ids: Vec<String> = stmt
        .query_map([], |r| r.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    let orders: Vec<Order> = ids.iter().filter_map(|id| load_order(&conn, id)).collect();
    Json(orders)
}

#[utoipa::path(post, path = "/orders", tag = "orders")]
pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateOrder>,
) -> Result<(StatusCode, Json<Order>), (StatusCode, String)> {
    let conn = state.db.lock().await;

    // Validate restaurant exists
    let rest_id_str = body.restaurant_id.to_string();
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM restaurants WHERE id = ?1",
            params![rest_id_str],
            |r| r.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    if !exists {
        return Err((StatusCode::NOT_FOUND, "restaurant not found".into()));
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

    // Insert order
    let id_str = order.id.to_string();
    let zone_str = order.zone_id.to_string();
    let created = order.created_at.to_rfc3339();
    let updated = order.updated_at.to_rfc3339();
    conn.execute(
        "INSERT INTO orders (id, restaurant_id, courier_id, customer_address, zone_id, status,
         food_total, delivery_fee, tip, federal_fee, local_ops_fee, processing_fee,
         created_at, updated_at, estimated_eta, actual_delivery_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![
            id_str,
            rest_id_str,
            Option::<String>::None,
            order.customer_address,
            zone_str,
            "Created",
            order.pricing.food_total.amount().to_string(),
            order.pricing.delivery_fee.amount().to_string(),
            order.pricing.tip.amount().to_string(),
            order.pricing.federal_fee.amount().to_string(),
            order.pricing.local_ops_fee.amount().to_string(),
            order.pricing.processing_fee.amount().to_string(),
            created,
            updated,
            order.estimated_eta,
            Option::<String>::None,
        ],
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Insert order items
    for item in &order.items {
        conn.execute(
            "INSERT INTO order_items (order_id, menu_item_id, name, quantity, unit_price)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id_str,
                item.menu_item_id.to_string(),
                item.name,
                item.quantity,
                item.unit_price.amount().to_string(),
            ],
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    let _ = state.order_events.send(OrderEvent {
        order_id: id_str,
        status: "Created".into(),
    });

    Ok((StatusCode::CREATED, Json(order)))
}

#[utoipa::path(get, path = "/orders/{id}", tag = "orders")]
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<OrderId>,
) -> Result<Json<Order>, StatusCode> {
    let conn = state.db.lock().await;
    load_order(&conn, &id.to_string())
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

#[utoipa::path(patch, path = "/orders/{id}/status", tag = "orders")]
pub async fn transition(
    State(state): State<AppState>,
    Path(id): Path<OrderId>,
    Json(body): Json<TransitionStatus>,
) -> Result<Json<Order>, (StatusCode, String)> {
    let conn = state.db.lock().await;
    let id_str = id.to_string();

    let mut order =
        load_order(&conn, &id_str).ok_or((StatusCode::NOT_FOUND, "order not found".into()))?;

    order
        .transition(body.status)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let updated = order.updated_at.to_rfc3339();
    conn.execute(
        "UPDATE orders SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![format!("{:?}", order.status), updated, id_str],
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let _ = state.order_events.send(OrderEvent {
        order_id: id_str,
        status: format!("{:?}", body.status),
    });

    Ok(Json(order))
}
