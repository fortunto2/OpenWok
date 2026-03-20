use std::collections::HashMap;

use openwok_core::money::Money;
use openwok_core::order::{Order, OrderItem, OrderStatus};
use openwok_core::pricing::PricingBreakdown;
use openwok_core::repo::{
    AdminMetrics, AssignCourierResult, CourierUtilization, CreateCourierRequest,
    CreateOrderRequest, CreateRestaurantRequest,
    PublicEconomics, RepoError, RevenueBreakdown,
};
use openwok_core::types::{
    Courier, CourierId, CourierKind, CreatePaymentRequest, CreateUserRequest, MenuItem, MenuItemId,
    OrderId, Payment, PaymentId, PaymentStatus, Restaurant, RestaurantId,
    UpdatePaymentStatusRequest, User, UserId, ZoneId,
};
use worker::d1::D1Database;

pub struct D1Repo {
    db: D1Database,
}

impl D1Repo {
    pub fn new(db: D1Database) -> Self {
        Self { db }
    }
}

// ── D1 row DTOs ────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct RestaurantRow {
    id: String,
    name: String,
    zone_id: String,
    active: i64,
}

#[derive(serde::Deserialize)]
struct MenuItemRow {
    id: String,
    name: String,
    price: String,
}

#[derive(serde::Deserialize)]
struct CourierRow {
    id: String,
    name: String,
    kind: String,
    zone_id: String,
    available: i64,
}

#[derive(serde::Deserialize)]
struct OrderRow {
    id: String,
    restaurant_id: String,
    courier_id: Option<String>,
    customer_address: String,
    zone_id: String,
    status: String,
    food_total: String,
    delivery_fee: String,
    tip: String,
    federal_fee: String,
    local_ops_fee: String,
    processing_fee: String,
    created_at: String,
    updated_at: String,
    estimated_eta: Option<f64>,
    actual_delivery_at: Option<String>,
}

#[derive(serde::Deserialize)]
struct OrderItemRow {
    menu_item_id: String,
    name: String,
    quantity: u32,
    unit_price: String,
}

// ── Helpers ────────────────────────────────────────────────────────────

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

fn parse_uuid(s: &str) -> uuid::Uuid {
    uuid::Uuid::parse_str(s).unwrap_or_else(|_| uuid::Uuid::nil())
}

fn row_to_restaurant(row: RestaurantRow, menu_rows: Vec<MenuItemRow>) -> Restaurant {
    let rid = RestaurantId::from_uuid(parse_uuid(&row.id));
    Restaurant {
        id: rid,
        name: row.name,
        zone_id: ZoneId::from_uuid(parse_uuid(&row.zone_id)),
        menu: menu_rows
            .into_iter()
            .map(|m| MenuItem {
                id: MenuItemId::from_uuid(parse_uuid(&m.id)),
                name: m.name,
                price: Money::from(m.price.as_str()),
                restaurant_id: rid,
            })
            .collect(),
        active: row.active != 0,
    }
}

fn row_to_order(row: OrderRow, items: Vec<OrderItemRow>) -> Option<Order> {
    let status = parse_status(&row.status)?;
    Some(Order {
        id: OrderId::from_uuid(parse_uuid(&row.id)),
        items: items
            .into_iter()
            .map(|i| OrderItem {
                menu_item_id: MenuItemId::from_uuid(parse_uuid(&i.menu_item_id)),
                name: i.name,
                quantity: i.quantity,
                unit_price: Money::from(i.unit_price.as_str()),
            })
            .collect(),
        restaurant_id: RestaurantId::from_uuid(parse_uuid(&row.restaurant_id)),
        courier_id: row
            .courier_id
            .as_deref()
            .map(|c| CourierId::from_uuid(parse_uuid(c))),
        customer_address: row.customer_address,
        zone_id: ZoneId::from_uuid(parse_uuid(&row.zone_id)),
        status,
        pricing: PricingBreakdown {
            food_total: Money::from(row.food_total.as_str()),
            delivery_fee: Money::from(row.delivery_fee.as_str()),
            tip: Money::from(row.tip.as_str()),
            federal_fee: Money::from(row.federal_fee.as_str()),
            local_ops_fee: Money::from(row.local_ops_fee.as_str()),
            processing_fee: Money::from(row.processing_fee.as_str()),
        },
        created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
            .unwrap()
            .with_timezone(&chrono::Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&row.updated_at)
            .unwrap()
            .with_timezone(&chrono::Utc),
        estimated_eta: row.estimated_eta.map(|v| v as i32),
        actual_delivery_at: row.actual_delivery_at.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        }),
    })
}

fn row_to_courier(row: CourierRow) -> Courier {
    Courier {
        id: CourierId::from_uuid(parse_uuid(&row.id)),
        name: row.name,
        kind: match row.kind.as_str() {
            "Human" => CourierKind::Human,
            _ => CourierKind::Human,
        },
        zone_id: ZoneId::from_uuid(parse_uuid(&row.zone_id)),
        available: row.available != 0,
    }
}

fn d1_err(e: worker::Error) -> RepoError {
    RepoError::Internal(e.to_string())
}

// ── D1 order loader ────────────────────────────────────────────────────

async fn load_order_d1(db: &D1Database, order_id: &str) -> Result<Option<Order>, RepoError> {
    let row = db
        .prepare(
            "SELECT id, restaurant_id, courier_id, customer_address, zone_id, status,
                    food_total, delivery_fee, tip, federal_fee, local_ops_fee, processing_fee,
                    created_at, updated_at, estimated_eta, actual_delivery_at
             FROM orders WHERE id = ?1",
        )
        .bind(&[order_id.into()])
        .map_err(d1_err)?
        .first::<OrderRow>(None)
        .await
        .map_err(d1_err)?;

    let row = match row {
        Some(r) => r,
        None => return Ok(None),
    };

    let items = db
        .prepare(
            "SELECT menu_item_id, name, quantity, unit_price FROM order_items WHERE order_id = ?1",
        )
        .bind(&[order_id.into()])
        .map_err(d1_err)?
        .all()
        .await
        .map_err(d1_err)?
        .results::<OrderItemRow>()
        .map_err(d1_err)?;

    Ok(row_to_order(row, items))
}

// ── D1Repo methods (same signatures as Repository trait) ───────────────
// D1Database is !Send, so we can't impl Repository directly.
// These methods are called by the worker's Router handlers.

impl D1Repo {
    pub async fn list_restaurants(&self) -> Result<Vec<Restaurant>, RepoError> {
        let rows = self
            .db
            .prepare("SELECT id, name, zone_id, active FROM restaurants WHERE active = 1")
            .all()
            .await
            .map_err(d1_err)?
            .results::<RestaurantRow>()
            .map_err(d1_err)?;

        let mut restaurants = Vec::new();
        for row in rows {
            let rid = row.id.clone();
            let menu = self
                .db
                .prepare("SELECT id, name, price FROM menu_items WHERE restaurant_id = ?1")
                .bind(&[rid.into()])
                .map_err(d1_err)?
                .all()
                .await
                .map_err(d1_err)?
                .results::<MenuItemRow>()
                .map_err(d1_err)?;
            restaurants.push(row_to_restaurant(row, menu));
        }
        Ok(restaurants)
    }

    pub async fn get_restaurant(&self, id: RestaurantId) -> Result<Restaurant, RepoError> {
        let id_str = id.to_string();
        let row = self
            .db
            .prepare("SELECT id, name, zone_id, active FROM restaurants WHERE id = ?1")
            .bind(&[id_str.clone().into()])
            .map_err(d1_err)?
            .first::<RestaurantRow>(None)
            .await
            .map_err(d1_err)?
            .ok_or(RepoError::NotFound)?;

        let menu = self
            .db
            .prepare("SELECT id, name, price FROM menu_items WHERE restaurant_id = ?1")
            .bind(&[id_str.into()])
            .map_err(d1_err)?
            .all()
            .await
            .map_err(d1_err)?
            .results::<MenuItemRow>()
            .map_err(d1_err)?;

        Ok(row_to_restaurant(row, menu))
    }

    pub async fn create_restaurant(
        &self,
        req: CreateRestaurantRequest,
    ) -> Result<Restaurant, RepoError> {
        let id = RestaurantId::new();
        let id_str = id.to_string();
        let zone_str = req.zone_id.to_string();

        self.db
            .prepare("INSERT OR IGNORE INTO zones (id, name) VALUES (?1, ?2)")
            .bind(&[
                zone_str.clone().into(),
                format!("Zone {}", &zone_str[..8.min(zone_str.len())]).into(),
            ])
            .map_err(d1_err)?
            .run()
            .await
            .map_err(d1_err)?;

        self.db
            .prepare(
                "INSERT INTO restaurants (id, name, zone_id, active) VALUES (?1, ?2, ?3, 1)",
            )
            .bind(&[
                id_str.clone().into(),
                req.name.clone().into(),
                zone_str.into(),
            ])
            .map_err(d1_err)?
            .run()
            .await
            .map_err(d1_err)?;

        let mut menu = Vec::new();
        for m in &req.menu {
            let mid = MenuItemId::new();
            let mid_str = mid.to_string();
            let price_str = m.price.amount().to_string();
            self.db
                .prepare("INSERT INTO menu_items (id, restaurant_id, name, price) VALUES (?1, ?2, ?3, ?4)")
                .bind(&[
                    mid_str.into(),
                    id_str.clone().into(),
                    m.name.clone().into(),
                    price_str.into(),
                ])
                .map_err(d1_err)?
                .run()
                .await
                .map_err(d1_err)?;
            menu.push(MenuItem {
                id: mid,
                name: m.name.clone(),
                price: m.price,
                restaurant_id: id,
            });
        }

        Ok(Restaurant {
            id,
            name: req.name,
            zone_id: req.zone_id,
            menu,
            active: true,
        })
    }

    pub async fn list_orders(&self) -> Result<Vec<Order>, RepoError> {
        let rows = self
            .db
            .prepare("SELECT id FROM orders")
            .all()
            .await
            .map_err(d1_err)?
            .results::<serde_json::Value>()
            .map_err(d1_err)?;

        let mut orders = Vec::new();
        for row in rows {
            let oid = row["id"].as_str().unwrap_or_default().to_string();
            if let Some(order) = load_order_d1(&self.db, &oid).await? {
                orders.push(order);
            }
        }
        Ok(orders)
    }

    pub async fn get_order(&self, id: OrderId) -> Result<Order, RepoError> {
        load_order_d1(&self.db, &id.to_string())
            .await?
            .ok_or(RepoError::NotFound)
    }

    pub async fn create_order(&self, req: CreateOrderRequest) -> Result<Order, RepoError> {
        let rest_id_str = req.restaurant_id.to_string();

        // Validate restaurant exists
        let exists = self
            .db
            .prepare("SELECT COUNT(*) as cnt FROM restaurants WHERE id = ?1")
            .bind(&[rest_id_str.clone().into()])
            .map_err(d1_err)?
            .first::<serde_json::Value>(None)
            .await
            .map_err(d1_err)?;
        let count = exists.and_then(|v| v["cnt"].as_i64()).unwrap_or(0);
        if count == 0 {
            return Err(RepoError::NotFound);
        }

        let items: Vec<OrderItem> = req
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
            req.restaurant_id,
            req.customer_address,
            req.zone_id,
            req.delivery_fee,
            req.tip,
            req.local_ops_fee,
        )
        .map_err(|e| RepoError::InvalidTransition(e.to_string()))?;

        let id_str = order.id.to_string();
        let zone_str = order.zone_id.to_string();
        let created = order.created_at.to_rfc3339();
        let updated = order.updated_at.to_rfc3339();

        self.db
            .prepare(
                "INSERT INTO orders (id, restaurant_id, courier_id, customer_address, zone_id, status,
                 food_total, delivery_fee, tip, federal_fee, local_ops_fee, processing_fee,
                 created_at, updated_at, estimated_eta, actual_delivery_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            )
            .bind(&[
                id_str.clone().into(),
                rest_id_str.into(),
                wasm_bindgen::JsValue::NULL,
                order.customer_address.clone().into(),
                zone_str.into(),
                "Created".into(),
                order.pricing.food_total.amount().to_string().into(),
                order.pricing.delivery_fee.amount().to_string().into(),
                order.pricing.tip.amount().to_string().into(),
                order.pricing.federal_fee.amount().to_string().into(),
                order.pricing.local_ops_fee.amount().to_string().into(),
                order.pricing.processing_fee.amount().to_string().into(),
                created.into(),
                updated.into(),
                wasm_bindgen::JsValue::NULL,
                wasm_bindgen::JsValue::NULL,
            ])
            .map_err(d1_err)?
            .run()
            .await
            .map_err(d1_err)?;

        for item in &order.items {
            self.db
                .prepare(
                    "INSERT INTO order_items (order_id, menu_item_id, name, quantity, unit_price)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                )
                .bind(&[
                    id_str.clone().into(),
                    item.menu_item_id.to_string().into(),
                    item.name.clone().into(),
                    wasm_bindgen::JsValue::from(item.quantity),
                    item.unit_price.amount().to_string().into(),
                ])
                .map_err(d1_err)?
                .run()
                .await
                .map_err(d1_err)?;
        }

        Ok(order)
    }

    pub async fn update_order_status(
        &self,
        id: OrderId,
        status: OrderStatus,
    ) -> Result<Order, RepoError> {
        let id_str = id.to_string();

        let order = load_order_d1(&self.db, &id_str)
            .await?
            .ok_or(RepoError::NotFound)?;

        // Validate transition
        if !order.status.valid_transitions().contains(&status) {
            return Err(RepoError::InvalidTransition(format!(
                "invalid transition from {:?} to {:?}",
                order.status, status
            )));
        }

        let now = chrono::Utc::now().to_rfc3339();
        self.db
            .prepare("UPDATE orders SET status = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(&[
                format!("{:?}", status).into(),
                now.into(),
                id_str.clone().into(),
            ])
            .map_err(d1_err)?
            .run()
            .await
            .map_err(d1_err)?;

        // Reload updated order
        load_order_d1(&self.db, &id_str)
            .await?
            .ok_or(RepoError::NotFound)
    }

    pub async fn assign_courier(&self, order_id: OrderId) -> Result<AssignCourierResult, RepoError> {
        let order_id_str = order_id.to_string();

        let zone_row = self
            .db
            .prepare("SELECT zone_id FROM orders WHERE id = ?1")
            .bind(&[order_id_str.clone().into()])
            .map_err(d1_err)?
            .first::<serde_json::Value>(None)
            .await
            .map_err(d1_err)?
            .ok_or(RepoError::NotFound)?;
        let zone_id = zone_row["zone_id"]
            .as_str()
            .ok_or_else(|| RepoError::Internal("invalid zone".into()))?
            .to_string();

        let courier_row = self
            .db
            .prepare("SELECT id FROM couriers WHERE available = 1 AND zone_id = ?1 LIMIT 1")
            .bind(&[zone_id.into()])
            .map_err(d1_err)?
            .first::<serde_json::Value>(None)
            .await
            .map_err(d1_err)?
            .ok_or_else(|| RepoError::Conflict("no available courier in zone".into()))?;
        let courier_id = courier_row["id"]
            .as_str()
            .ok_or_else(|| RepoError::Internal("invalid courier".into()))?
            .to_string();

        self.db
            .prepare("UPDATE couriers SET available = 0 WHERE id = ?1")
            .bind(&[courier_id.clone().into()])
            .map_err(d1_err)?
            .run()
            .await
            .map_err(d1_err)?;

        self.db
            .prepare("UPDATE orders SET courier_id = ?1 WHERE id = ?2")
            .bind(&[courier_id.clone().into(), order_id_str.clone().into()])
            .map_err(d1_err)?
            .run()
            .await
            .map_err(d1_err)?;

        Ok(AssignCourierResult {
            order_id: order_id_str,
            courier_id,
        })
    }

    pub async fn list_couriers(&self) -> Result<Vec<Courier>, RepoError> {
        let rows = self
            .db
            .prepare("SELECT id, name, kind, zone_id, available FROM couriers WHERE available = 1")
            .all()
            .await
            .map_err(d1_err)?
            .results::<CourierRow>()
            .map_err(d1_err)?;
        Ok(rows.into_iter().map(row_to_courier).collect())
    }

    pub async fn create_courier(&self, req: CreateCourierRequest) -> Result<Courier, RepoError> {
        let id = CourierId::new();
        let id_str = id.to_string();
        let zone_str = req.zone_id.to_string();

        self.db
            .prepare("INSERT OR IGNORE INTO zones (id, name) VALUES (?1, ?2)")
            .bind(&[
                zone_str.clone().into(),
                format!("Zone {}", &zone_str[..8.min(zone_str.len())]).into(),
            ])
            .map_err(d1_err)?
            .run()
            .await
            .map_err(d1_err)?;

        self.db
            .prepare(
                "INSERT INTO couriers (id, name, kind, zone_id, available) VALUES (?1, ?2, 'Human', ?3, 1)",
            )
            .bind(&[
                id_str.into(),
                req.name.clone().into(),
                zone_str.into(),
            ])
            .map_err(d1_err)?
            .run()
            .await
            .map_err(d1_err)?;

        Ok(Courier {
            id,
            name: req.name,
            kind: CourierKind::Human,
            zone_id: req.zone_id,
            available: true,
        })
    }

    pub async fn toggle_courier_available(
        &self,
        id: CourierId,
        available: bool,
    ) -> Result<Courier, RepoError> {
        let id_str = id.to_string();
        let avail_val: i64 = if available { 1 } else { 0 };

        self.db
            .prepare("UPDATE couriers SET available = ?1 WHERE id = ?2")
            .bind(&[
                wasm_bindgen::JsValue::from(avail_val),
                id_str.clone().into(),
            ])
            .map_err(d1_err)?
            .run()
            .await
            .map_err(d1_err)?;

        let row = self
            .db
            .prepare("SELECT id, name, kind, zone_id, available FROM couriers WHERE id = ?1")
            .bind(&[id_str.into()])
            .map_err(d1_err)?
            .first::<CourierRow>(None)
            .await
            .map_err(d1_err)?
            .ok_or(RepoError::NotFound)?;

        Ok(row_to_courier(row))
    }

    pub async fn get_economics(&self) -> Result<PublicEconomics, RepoError> {
        let row = self
            .db
            .prepare(
                "SELECT
                    COUNT(*) as total_orders,
                    COALESCE(SUM(CAST(food_total AS REAL)), 0) as total_food_revenue,
                    COALESCE(SUM(CAST(delivery_fee AS REAL)), 0) as total_delivery_fees,
                    COALESCE(SUM(CAST(federal_fee AS REAL)), 0) as total_federal_fees,
                    COALESCE(SUM(CAST(local_ops_fee AS REAL)), 0) as total_local_ops_fees,
                    COALESCE(SUM(CAST(processing_fee AS REAL)), 0) as total_processing_fees,
                    CASE WHEN COUNT(*) > 0
                        THEN (COALESCE(SUM(CAST(food_total AS REAL)), 0)
                            + COALESCE(SUM(CAST(delivery_fee AS REAL)), 0)
                            + COALESCE(SUM(CAST(federal_fee AS REAL)), 0)
                            + COALESCE(SUM(CAST(local_ops_fee AS REAL)), 0)
                            + COALESCE(SUM(CAST(processing_fee AS REAL)), 0))
                            / COUNT(*)
                        ELSE 0
                    END as avg_order_value
                 FROM orders",
            )
            .first::<serde_json::Value>(None)
            .await
            .map_err(d1_err)?
            .unwrap_or(serde_json::json!({}));

        Ok(PublicEconomics {
            total_orders: row["total_orders"].as_i64().unwrap_or(0),
            total_food_revenue: format!(
                "{:.2}",
                row["total_food_revenue"].as_f64().unwrap_or(0.0)
            ),
            total_delivery_fees: format!(
                "{:.2}",
                row["total_delivery_fees"].as_f64().unwrap_or(0.0)
            ),
            total_federal_fees: format!(
                "{:.2}",
                row["total_federal_fees"].as_f64().unwrap_or(0.0)
            ),
            total_local_ops_fees: format!(
                "{:.2}",
                row["total_local_ops_fees"].as_f64().unwrap_or(0.0)
            ),
            total_processing_fees: format!(
                "{:.2}",
                row["total_processing_fees"].as_f64().unwrap_or(0.0)
            ),
            avg_order_value: format!("{:.2}", row["avg_order_value"].as_f64().unwrap_or(0.0)),
        })
    }

    pub async fn get_metrics(&self) -> Result<AdminMetrics, RepoError> {
        let count_row = self
            .db
            .prepare("SELECT COUNT(*) as cnt FROM orders")
            .first::<serde_json::Value>(None)
            .await
            .map_err(d1_err)?;
        let order_count = count_row.and_then(|v| v["cnt"].as_i64()).unwrap_or(0);

        let mut orders_by_status = HashMap::new();
        let status_rows = self
            .db
            .prepare("SELECT status, COUNT(*) as cnt FROM orders GROUP BY status")
            .all()
            .await
            .map_err(d1_err)?
            .results::<serde_json::Value>()
            .map_err(d1_err)?;
        for row in status_rows {
            if let (Some(s), Some(c)) = (row["status"].as_str(), row["cnt"].as_i64()) {
                orders_by_status.insert(s.to_string(), c);
            }
        }

        let otd = self
            .db
            .prepare(
                "SELECT
                    SUM(CASE WHEN (julianday(actual_delivery_at) - julianday(created_at)) * 1440 < estimated_eta THEN 1 ELSE 0 END) as on_time,
                    COUNT(*) as total
                 FROM orders WHERE actual_delivery_at IS NOT NULL AND estimated_eta IS NOT NULL",
            )
            .first::<serde_json::Value>(None)
            .await
            .map_err(d1_err)?
            .unwrap_or(serde_json::json!({}));
        let on_time = otd["on_time"].as_i64().unwrap_or(0);
        let total_delivered = otd["total"].as_i64().unwrap_or(0);
        let on_time_delivery_rate = if total_delivered > 0 {
            on_time as f64 / total_delivered as f64 * 100.0
        } else {
            0.0
        };

        let eta_row = self
            .db
            .prepare(
                "SELECT COALESCE(AVG(ABS((julianday(actual_delivery_at) - julianday(created_at)) * 1440 - estimated_eta)), 0) as avg_err
                 FROM orders WHERE actual_delivery_at IS NOT NULL AND estimated_eta IS NOT NULL",
            )
            .first::<serde_json::Value>(None)
            .await
            .map_err(d1_err)?;
        let avg_eta_error_minutes = eta_row
            .and_then(|v| v["avg_err"].as_f64())
            .unwrap_or(0.0);

        let rev = self
            .db
            .prepare(
                "SELECT
                    COALESCE(SUM(CAST(food_total AS REAL)), 0) as f,
                    COALESCE(SUM(CAST(delivery_fee AS REAL)), 0) as d,
                    COALESCE(SUM(CAST(federal_fee AS REAL)), 0) as fed,
                    COALESCE(SUM(CAST(local_ops_fee AS REAL)), 0) as loc,
                    COALESCE(SUM(CAST(processing_fee AS REAL)), 0) as proc
                 FROM orders",
            )
            .first::<serde_json::Value>(None)
            .await
            .map_err(d1_err)?
            .unwrap_or(serde_json::json!({}));

        let cu = self
            .db
            .prepare("SELECT SUM(CASE WHEN available=1 THEN 1 ELSE 0 END) as avail, COUNT(*) as total FROM couriers")
            .first::<serde_json::Value>(None)
            .await
            .map_err(d1_err)?
            .unwrap_or(serde_json::json!({}));

        let mut orders_by_zone = HashMap::new();
        let zone_rows = self
            .db
            .prepare("SELECT z.name, COUNT(o.id) as cnt FROM orders o JOIN zones z ON o.zone_id = z.id GROUP BY z.name")
            .all()
            .await
            .map_err(d1_err)?
            .results::<serde_json::Value>()
            .map_err(d1_err)?;
        for row in zone_rows {
            if let (Some(n), Some(c)) = (row["name"].as_str(), row["cnt"].as_i64()) {
                orders_by_zone.insert(n.to_string(), c);
            }
        }

        Ok(AdminMetrics {
            order_count,
            orders_by_status,
            on_time_delivery_rate,
            avg_eta_error_minutes,
            revenue_breakdown: RevenueBreakdown {
                total_food_revenue: format!("{:.2}", rev["f"].as_f64().unwrap_or(0.0)),
                total_delivery_fees: format!("{:.2}", rev["d"].as_f64().unwrap_or(0.0)),
                total_federal_fees: format!("{:.2}", rev["fed"].as_f64().unwrap_or(0.0)),
                total_local_ops_fees: format!("{:.2}", rev["loc"].as_f64().unwrap_or(0.0)),
                total_processing_fees: format!("{:.2}", rev["proc"].as_f64().unwrap_or(0.0)),
            },
            courier_utilization: CourierUtilization {
                available: cu["avail"].as_i64().unwrap_or(0),
                total: cu["total"].as_i64().unwrap_or(0),
            },
            orders_by_zone,
        })
    }

    // ── User methods ──────────────────────────────────────────────────

    pub async fn create_user(&self, req: CreateUserRequest) -> Result<User, RepoError> {
        let id = UserId::new();
        let id_str = id.to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let role_enum = req.role.unwrap_or(openwok_core::types::UserRole::Customer);
        let role = format!("{:?}", role_enum);

        self.db
            .prepare(
                "INSERT INTO users (id, supabase_user_id, email, name, role, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(&[
                id_str.clone().into(),
                req.supabase_user_id.clone().into(),
                req.email.clone().into(),
                req.name.clone().unwrap_or_default().into(),
                role.into(),
                now.clone().into(),
            ])
            .map_err(d1_err)?
            .run()
            .await
            .map_err(d1_err)?;

        Ok(User {
            id,
            supabase_user_id: req.supabase_user_id,
            email: req.email,
            name: req.name,
            role: role_enum,
            created_at: chrono::DateTime::parse_from_rfc3339(&now)
                .unwrap()
                .with_timezone(&chrono::Utc),
        })
    }

    pub async fn get_user_by_supabase_id(&self, supabase_id: &str) -> Result<User, RepoError> {
        let row = self
            .db
            .prepare("SELECT id, supabase_user_id, email, name, role, created_at FROM users WHERE supabase_user_id = ?1")
            .bind(&[supabase_id.into()])
            .map_err(d1_err)?
            .first::<serde_json::Value>(None)
            .await
            .map_err(d1_err)?
            .ok_or(RepoError::NotFound)?;

        Ok(User {
            id: UserId::from_uuid(parse_uuid(row["id"].as_str().unwrap_or_default())),
            supabase_user_id: row["supabase_user_id"].as_str().unwrap_or_default().to_string(),
            email: row["email"].as_str().unwrap_or_default().to_string(),
            name: row["name"].as_str().map(|s| s.to_string()),
            role: row["role"]
                .as_str()
                .unwrap_or("Customer")
                .parse()
                .unwrap_or(openwok_core::types::UserRole::Customer),
            created_at: chrono::DateTime::parse_from_rfc3339(
                row["created_at"].as_str().unwrap_or_default(),
            )
            .unwrap_or_default()
            .with_timezone(&chrono::Utc),
        })
    }

    // ── Payment methods ───────────────────────────────────────────────

    pub async fn create_payment(&self, req: CreatePaymentRequest) -> Result<Payment, RepoError> {
        let id = PaymentId::new();
        let id_str = id.to_string();
        let order_id_str = req.order_id.to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.db
            .prepare(
                "INSERT INTO payments (id, order_id, stripe_payment_intent_id, stripe_checkout_session_id,
                 status, amount_total, restaurant_amount, courier_amount, federal_amount,
                 local_ops_amount, processing_amount, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            )
            .bind(&[
                id_str.into(),
                order_id_str.into(),
                wasm_bindgen::JsValue::NULL,
                req.stripe_checkout_session_id.clone().map_or(wasm_bindgen::JsValue::NULL, |s: String| s.into()),
                "Pending".into(),
                req.amount_total.amount().to_string().into(),
                req.restaurant_amount.amount().to_string().into(),
                req.courier_amount.amount().to_string().into(),
                req.federal_amount.amount().to_string().into(),
                req.local_ops_amount.amount().to_string().into(),
                req.processing_amount.amount().to_string().into(),
                now.clone().into(),
            ])
            .map_err(d1_err)?
            .run()
            .await
            .map_err(d1_err)?;

        Ok(Payment {
            id,
            order_id: req.order_id,
            stripe_payment_intent_id: None,
            stripe_checkout_session_id: req.stripe_checkout_session_id,
            status: PaymentStatus::Pending,
            amount_total: req.amount_total,
            restaurant_amount: req.restaurant_amount,
            courier_amount: req.courier_amount,
            federal_amount: req.federal_amount,
            local_ops_amount: req.local_ops_amount,
            processing_amount: req.processing_amount,
            created_at: chrono::DateTime::parse_from_rfc3339(&now)
                .unwrap()
                .with_timezone(&chrono::Utc),
        })
    }

    pub async fn get_payment_by_order(&self, order_id: OrderId) -> Result<Payment, RepoError> {
        let row = self
            .db
            .prepare("SELECT id, order_id, stripe_payment_intent_id, stripe_checkout_session_id, status, amount_total, restaurant_amount, courier_amount, federal_amount, local_ops_amount, processing_amount, created_at FROM payments WHERE order_id = ?1")
            .bind(&[order_id.to_string().into()])
            .map_err(d1_err)?
            .first::<serde_json::Value>(None)
            .await
            .map_err(d1_err)?
            .ok_or(RepoError::NotFound)?;

        Ok(Payment {
            id: PaymentId::from_uuid(parse_uuid(row["id"].as_str().unwrap_or_default())),
            order_id,
            stripe_payment_intent_id: row["stripe_payment_intent_id"].as_str().map(|s| s.to_string()),
            stripe_checkout_session_id: row["stripe_checkout_session_id"].as_str().map(|s| s.to_string()),
            status: row["status"].as_str().unwrap_or("Pending").parse().unwrap_or(PaymentStatus::Pending),
            amount_total: Money::from(row["amount_total"].as_str().unwrap_or("0")),
            restaurant_amount: Money::from(row["restaurant_amount"].as_str().unwrap_or("0")),
            courier_amount: Money::from(row["courier_amount"].as_str().unwrap_or("0")),
            federal_amount: Money::from(row["federal_amount"].as_str().unwrap_or("0")),
            local_ops_amount: Money::from(row["local_ops_amount"].as_str().unwrap_or("0")),
            processing_amount: Money::from(row["processing_amount"].as_str().unwrap_or("0")),
            created_at: chrono::DateTime::parse_from_rfc3339(row["created_at"].as_str().unwrap_or_default())
                .unwrap_or_default()
                .with_timezone(&chrono::Utc),
        })
    }

    pub async fn update_payment_status(
        &self,
        id: PaymentId,
        req: UpdatePaymentStatusRequest,
    ) -> Result<(), RepoError> {
        let status_str = format!("{:?}", req.status);
        let id_str = id.to_string();

        if let Some(ref pi_id) = req.stripe_payment_intent_id {
            self.db
                .prepare("UPDATE payments SET status = ?1, stripe_payment_intent_id = ?2 WHERE id = ?3")
                .bind(&[status_str.into(), pi_id.clone().into(), id_str.into()])
                .map_err(d1_err)?
                .run()
                .await
                .map_err(d1_err)?;
        } else {
            self.db
                .prepare("UPDATE payments SET status = ?1 WHERE id = ?2")
                .bind(&[status_str.into(), id_str.into()])
                .map_err(d1_err)?
                .run()
                .await
                .map_err(d1_err)?;
        }
        Ok(())
    }
}
