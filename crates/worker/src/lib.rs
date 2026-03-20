use openwok_core::money::Money;
use openwok_core::order::OrderStatus;
use openwok_core::pricing::calculate_pricing;
use serde::{Deserialize, Serialize};
use worker::*;

// ── D1 row DTOs ────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RestaurantRow {
    id: String,
    name: String,
    zone_id: String,
    active: i64,
}

#[derive(Deserialize, Serialize, Clone)]
struct MenuItemRow {
    id: String,
    name: String,
    price: String,
}

#[derive(Deserialize)]
struct CourierRow {
    id: String,
    name: String,
    kind: String,
    zone_id: String,
    available: i64,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
struct OrderItemRow {
    menu_item_id: String,
    name: String,
    quantity: u32,
    unit_price: String,
}

// ── Response types (String IDs — compatible with any ID format in DB) ──

#[derive(Serialize)]
struct RestaurantResp {
    id: String,
    name: String,
    zone_id: String,
    active: bool,
    menu: Vec<MenuItemResp>,
}

#[derive(Serialize)]
struct MenuItemResp {
    id: String,
    name: String,
    price: String,
    restaurant_id: String,
}

#[derive(Serialize)]
struct CourierResp {
    id: String,
    name: String,
    kind: String,
    zone_id: String,
    available: bool,
}

#[derive(Serialize)]
struct PricingResp {
    food_total: String,
    delivery_fee: String,
    tip: String,
    federal_fee: String,
    local_ops_fee: String,
    processing_fee: String,
}

#[derive(Serialize)]
struct OrderResp {
    id: String,
    items: Vec<OrderItemResp>,
    restaurant_id: String,
    courier_id: Option<String>,
    customer_address: String,
    zone_id: String,
    status: String,
    pricing: PricingResp,
    created_at: String,
    updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    estimated_eta: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_delivery_at: Option<String>,
}

#[derive(Serialize)]
struct OrderItemResp {
    menu_item_id: String,
    name: String,
    quantity: u32,
    unit_price: String,
}

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

// ── Aggregate response types ───────────────────────────────────────────

#[derive(Serialize)]
struct PublicEconomics {
    total_orders: i64,
    total_food_revenue: String,
    total_delivery_fees: String,
    total_federal_fees: String,
    total_local_ops_fees: String,
    total_processing_fees: String,
    avg_order_value: String,
}

#[derive(Serialize)]
struct AdminMetrics {
    order_count: i64,
    orders_by_status: std::collections::HashMap<String, i64>,
    on_time_delivery_rate: f64,
    avg_eta_error_minutes: f64,
    revenue_breakdown: RevenueBreakdown,
    courier_utilization: CourierUtil,
    orders_by_zone: std::collections::HashMap<String, i64>,
}

#[derive(Serialize)]
struct RevenueBreakdown {
    total_food_revenue: String,
    total_delivery_fees: String,
    total_federal_fees: String,
    total_local_ops_fees: String,
    total_processing_fees: String,
}

#[derive(Serialize)]
struct CourierUtil {
    available: i64,
    total: i64,
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

fn to_restaurant_resp(row: RestaurantRow, menu_rows: Vec<MenuItemRow>) -> RestaurantResp {
    let rid = row.id.clone();
    RestaurantResp {
        id: row.id,
        name: row.name,
        zone_id: row.zone_id,
        active: row.active != 0,
        menu: menu_rows
            .into_iter()
            .map(|m| MenuItemResp {
                id: m.id,
                name: m.name,
                price: m.price,
                restaurant_id: rid.clone(),
            })
            .collect(),
    }
}

fn to_order_resp(row: OrderRow, items: Vec<OrderItemRow>) -> OrderResp {
    OrderResp {
        id: row.id,
        items: items
            .into_iter()
            .map(|i| OrderItemResp {
                menu_item_id: i.menu_item_id,
                name: i.name,
                quantity: i.quantity,
                unit_price: i.unit_price,
            })
            .collect(),
        restaurant_id: row.restaurant_id,
        courier_id: row.courier_id,
        customer_address: row.customer_address,
        zone_id: row.zone_id,
        status: row.status,
        pricing: PricingResp {
            food_total: row.food_total,
            delivery_fee: row.delivery_fee,
            tip: row.tip,
            federal_fee: row.federal_fee,
            local_ops_fee: row.local_ops_fee,
            processing_fee: row.processing_fee,
        },
        created_at: row.created_at,
        updated_at: row.updated_at,
        estimated_eta: row.estimated_eta.map(|v| v as i32),
        actual_delivery_at: row.actual_delivery_at,
    }
}

fn to_courier_resp(row: CourierRow) -> CourierResp {
    CourierResp {
        id: row.id,
        name: row.name,
        kind: row.kind,
        zone_id: row.zone_id,
        available: row.available != 0,
    }
}

fn json_response<T: Serialize>(data: &T, status: u16) -> Result<Response> {
    let body = serde_json::to_string(data).map_err(|e| Error::RustError(e.to_string()))?;
    let mut resp = Response::ok(body)?;
    resp = resp.with_status(status);
    resp.headers_mut().set("Content-Type", "application/json")?;
    Ok(resp)
}

fn error_response(msg: &str, status: u16) -> Result<Response> {
    Response::error(msg, status)
}

// ── Entry point ────────────────────────────────────────────────────────

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        // Health
        .get_async("/api/health", |_, _| async move { Response::ok("ok") })
        // Restaurants
        .get_async("/api/restaurants", |_, ctx| async move {
            let d1 = ctx.env.d1("DB")?;
            let rows = d1
                .prepare("SELECT id, name, zone_id, active FROM restaurants WHERE active = 1")
                .all()
                .await?
                .results::<RestaurantRow>()?;

            let mut restaurants = Vec::new();
            for row in rows {
                let rid = row.id.clone();
                let menu = d1
                    .prepare("SELECT id, name, price FROM menu_items WHERE restaurant_id = ?1")
                    .bind(&[rid.into()])?
                    .all()
                    .await?
                    .results::<MenuItemRow>()?;
                restaurants.push(to_restaurant_resp(row, menu));
            }
            json_response(&restaurants, 200)
        })
        .get_async("/api/restaurants/:id", |_, ctx| async move {
            let id = ctx.param("id").unwrap().to_string();
            let d1 = ctx.env.d1("DB")?;
            let row = d1
                .prepare("SELECT id, name, zone_id, active FROM restaurants WHERE id = ?1")
                .bind(&[id.clone().into()])?
                .first::<RestaurantRow>(None)
                .await?;
            match row {
                Some(row) => {
                    let rid = row.id.clone();
                    let menu = d1
                        .prepare(
                            "SELECT id, name, price FROM menu_items WHERE restaurant_id = ?1",
                        )
                        .bind(&[rid.into()])?
                        .all()
                        .await?
                        .results::<MenuItemRow>()?;
                    json_response(&to_restaurant_resp(row, menu), 200)
                }
                None => error_response("not found", 404),
            }
        })
        .post_async("/api/restaurants", |mut req, ctx| async move {
            let body: CreateRestaurantReq = req.json().await?;
            let d1 = ctx.env.d1("DB")?;
            let id = uuid::Uuid::new_v4().to_string();

            // Ensure zone exists
            d1.prepare("INSERT OR IGNORE INTO zones (id, name) VALUES (?1, ?2)")
                .bind(&[
                    body.zone_id.clone().into(),
                    format!("Zone {}", &body.zone_id[..8.min(body.zone_id.len())]).into(),
                ])?
                .run()
                .await?;

            d1.prepare(
                "INSERT INTO restaurants (id, name, zone_id, active) VALUES (?1, ?2, ?3, 1)",
            )
            .bind(&[id.clone().into(), body.name.clone().into(), body.zone_id.clone().into()])?
            .run()
            .await?;

            let mut menu = Vec::new();
            for m in &body.menu {
                let mid = uuid::Uuid::new_v4().to_string();
                d1.prepare("INSERT INTO menu_items (id, restaurant_id, name, price) VALUES (?1, ?2, ?3, ?4)")
                    .bind(&[mid.clone().into(), id.clone().into(), m.name.clone().into(), m.price.clone().into()])?
                    .run().await?;
                menu.push(MenuItemResp {
                    id: mid,
                    name: m.name.clone(),
                    price: m.price.clone(),
                    restaurant_id: id.clone(),
                });
            }

            let resp = RestaurantResp {
                id,
                name: body.name,
                zone_id: body.zone_id,
                active: true,
                menu,
            };
            json_response(&resp, 201)
        })
        // Orders
        .get_async("/api/orders", |_, ctx| async move {
            let d1 = ctx.env.d1("DB")?;
            let rows = d1
                .prepare("SELECT id FROM orders")
                .all()
                .await?
                .results::<serde_json::Value>()?;

            let mut orders = Vec::new();
            for row in rows {
                let oid = row["id"].as_str().unwrap_or_default().to_string();
                if let Some(order) = load_order_d1(&d1, &oid).await? {
                    orders.push(order);
                }
            }
            json_response(&orders, 200)
        })
        .post_async("/api/orders", |mut req, ctx| async move {
            let body: CreateOrderReq = req.json().await?;
            let d1 = ctx.env.d1("DB")?;

            // Validate restaurant exists
            let exists = d1
                .prepare("SELECT COUNT(*) as cnt FROM restaurants WHERE id = ?1")
                .bind(&[body.restaurant_id.clone().into()])?
                .first::<serde_json::Value>(None)
                .await?;
            let count = exists.and_then(|v| v["cnt"].as_i64()).unwrap_or(0);
            if count == 0 {
                return error_response("restaurant not found", 404);
            }

            // Calculate pricing using core logic
            let food_total = body.items.iter().fold(Money::zero(), |acc, item| {
                let price = Money::from(item.unit_price.as_str());
                acc + price * rust_decimal::Decimal::from(item.quantity)
            });
            let pricing = calculate_pricing(
                food_total,
                Money::from(body.delivery_fee.as_str()),
                Money::from(body.tip.as_str()),
                Money::from(body.local_ops_fee.as_str()),
            );

            let id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();

            d1.prepare(
                "INSERT INTO orders (id, restaurant_id, courier_id, customer_address, zone_id, status,
                 food_total, delivery_fee, tip, federal_fee, local_ops_fee, processing_fee,
                 created_at, updated_at, estimated_eta, actual_delivery_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            )
            .bind(&[
                id.clone().into(),
                body.restaurant_id.clone().into(),
                wasm_bindgen::JsValue::NULL,
                body.customer_address.clone().into(),
                body.zone_id.clone().into(),
                "Created".into(),
                pricing.food_total.amount().to_string().into(),
                pricing.delivery_fee.amount().to_string().into(),
                pricing.tip.amount().to_string().into(),
                pricing.federal_fee.amount().to_string().into(),
                pricing.local_ops_fee.amount().to_string().into(),
                pricing.processing_fee.amount().to_string().into(),
                now.clone().into(),
                now.clone().into(),
                wasm_bindgen::JsValue::NULL,
                wasm_bindgen::JsValue::NULL,
            ])?
            .run()
            .await?;

            for item in &body.items {
                d1.prepare(
                    "INSERT INTO order_items (order_id, menu_item_id, name, quantity, unit_price)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                )
                .bind(&[
                    id.clone().into(),
                    item.menu_item_id.clone().into(),
                    item.name.clone().into(),
                    wasm_bindgen::JsValue::from(item.quantity),
                    item.unit_price.clone().into(),
                ])?
                .run()
                .await?;
            }

            let order = OrderResp {
                id,
                items: body
                    .items
                    .into_iter()
                    .map(|i| OrderItemResp {
                        menu_item_id: i.menu_item_id,
                        name: i.name,
                        quantity: i.quantity,
                        unit_price: i.unit_price,
                    })
                    .collect(),
                restaurant_id: body.restaurant_id,
                courier_id: None,
                customer_address: body.customer_address,
                zone_id: body.zone_id,
                status: "Created".into(),
                pricing: PricingResp {
                    food_total: pricing.food_total.amount().to_string(),
                    delivery_fee: pricing.delivery_fee.amount().to_string(),
                    tip: pricing.tip.amount().to_string(),
                    federal_fee: pricing.federal_fee.amount().to_string(),
                    local_ops_fee: pricing.local_ops_fee.amount().to_string(),
                    processing_fee: pricing.processing_fee.amount().to_string(),
                },
                created_at: now.clone(),
                updated_at: now,
                estimated_eta: None,
                actual_delivery_at: None,
            };
            json_response(&order, 201)
        })
        .get_async("/api/orders/:id", |_, ctx| async move {
            let id = ctx.param("id").unwrap().to_string();
            let d1 = ctx.env.d1("DB")?;
            match load_order_d1(&d1, &id).await? {
                Some(order) => json_response(&order, 200),
                None => error_response("not found", 404),
            }
        })
        .patch_async("/api/orders/:id/status", |mut req, ctx| async move {
            let id = ctx.param("id").unwrap().to_string();
            let body: TransitionReq = req.json().await?;
            let d1 = ctx.env.d1("DB")?;

            let order = load_order_d1(&d1, &id)
                .await?
                .ok_or_else(|| Error::RustError("order not found".into()))?;

            let current = parse_status(&order.status)
                .ok_or_else(|| Error::RustError("invalid current status".into()))?;
            let new_status = parse_status(&body.status)
                .ok_or_else(|| Error::RustError("invalid status".into()))?;

            // Validate transition
            if !current.valid_transitions().contains(&new_status) {
                return error_response(
                    &format!("invalid transition from {:?} to {:?}", current, new_status),
                    400,
                );
            }

            let now = chrono::Utc::now().to_rfc3339();
            d1.prepare("UPDATE orders SET status = ?1, updated_at = ?2 WHERE id = ?3")
                .bind(&[
                    format!("{:?}", new_status).into(),
                    now.into(),
                    id.clone().into(),
                ])?
                .run()
                .await?;

            // Reload and return updated order
            let updated = load_order_d1(&d1, &id)
                .await?
                .ok_or_else(|| Error::RustError("order not found after update".into()))?;
            json_response(&updated, 200)
        })
        // Courier assignment
        .post_async("/api/orders/:id/assign", |_, ctx| async move {
            let order_id = ctx.param("id").unwrap().to_string();
            let d1 = ctx.env.d1("DB")?;

            let zone_row = d1
                .prepare("SELECT zone_id FROM orders WHERE id = ?1")
                .bind(&[order_id.clone().into()])?
                .first::<serde_json::Value>(None)
                .await?
                .ok_or_else(|| Error::RustError("order not found".into()))?;
            let zone_id = zone_row["zone_id"]
                .as_str()
                .ok_or_else(|| Error::RustError("invalid zone".into()))?
                .to_string();

            let courier_row = d1
                .prepare("SELECT id FROM couriers WHERE available = 1 AND zone_id = ?1 LIMIT 1")
                .bind(&[zone_id.into()])?
                .first::<serde_json::Value>(None)
                .await?
                .ok_or_else(|| Error::RustError("no available courier in zone".into()))?;
            let courier_id = courier_row["id"]
                .as_str()
                .ok_or_else(|| Error::RustError("invalid courier".into()))?
                .to_string();

            d1.prepare("UPDATE couriers SET available = 0 WHERE id = ?1")
                .bind(&[courier_id.clone().into()])?
                .run()
                .await?;

            d1.prepare("UPDATE orders SET courier_id = ?1 WHERE id = ?2")
                .bind(&[courier_id.clone().into(), order_id.clone().into()])?
                .run()
                .await?;

            json_response(
                &serde_json::json!({"order_id": order_id, "courier_id": courier_id}),
                200,
            )
        })
        // Couriers
        .get_async("/api/couriers", |_, ctx| async move {
            let d1 = ctx.env.d1("DB")?;
            let rows = d1
                .prepare(
                    "SELECT id, name, kind, zone_id, available FROM couriers WHERE available = 1",
                )
                .all()
                .await?
                .results::<CourierRow>()?;
            let couriers: Vec<CourierResp> = rows.into_iter().map(to_courier_resp).collect();
            json_response(&couriers, 200)
        })
        .post_async("/api/couriers", |mut req, ctx| async move {
            let body: CreateCourierReq = req.json().await?;
            let d1 = ctx.env.d1("DB")?;
            let id = uuid::Uuid::new_v4().to_string();

            d1.prepare("INSERT OR IGNORE INTO zones (id, name) VALUES (?1, ?2)")
                .bind(&[
                    body.zone_id.clone().into(),
                    format!("Zone {}", &body.zone_id[..8.min(body.zone_id.len())]).into(),
                ])?
                .run()
                .await?;

            d1.prepare(
                "INSERT INTO couriers (id, name, kind, zone_id, available) VALUES (?1, ?2, 'Human', ?3, 1)",
            )
            .bind(&[id.clone().into(), body.name.clone().into(), body.zone_id.clone().into()])?
            .run()
            .await?;

            json_response(
                &CourierResp {
                    id,
                    name: body.name,
                    kind: "Human".into(),
                    zone_id: body.zone_id,
                    available: true,
                },
                201,
            )
        })
        .patch_async("/api/couriers/:id/available", |mut req, ctx| async move {
            let id = ctx.param("id").unwrap().to_string();
            let body: SetAvailableReq = req.json().await?;
            let d1 = ctx.env.d1("DB")?;

            let available: i64 = if body.available { 1 } else { 0 };
            d1.prepare("UPDATE couriers SET available = ?1 WHERE id = ?2")
                .bind(&[wasm_bindgen::JsValue::from(available), id.clone().into()])?
                .run()
                .await?;

            let row = d1
                .prepare("SELECT id, name, kind, zone_id, available FROM couriers WHERE id = ?1")
                .bind(&[id.into()])?
                .first::<CourierRow>(None)
                .await?
                .ok_or_else(|| Error::RustError("courier not found".into()))?;

            json_response(&to_courier_resp(row), 200)
        })
        // Economics (public)
        .get_async("/api/public/economics", |_, ctx| async move {
            let d1 = ctx.env.d1("DB")?;
            let row = d1
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
                .await?
                .unwrap_or(serde_json::json!({}));

            let economics = PublicEconomics {
                total_orders: row["total_orders"].as_i64().unwrap_or(0),
                total_food_revenue: format!("{:.2}", row["total_food_revenue"].as_f64().unwrap_or(0.0)),
                total_delivery_fees: format!("{:.2}", row["total_delivery_fees"].as_f64().unwrap_or(0.0)),
                total_federal_fees: format!("{:.2}", row["total_federal_fees"].as_f64().unwrap_or(0.0)),
                total_local_ops_fees: format!("{:.2}", row["total_local_ops_fees"].as_f64().unwrap_or(0.0)),
                total_processing_fees: format!("{:.2}", row["total_processing_fees"].as_f64().unwrap_or(0.0)),
                avg_order_value: format!("{:.2}", row["avg_order_value"].as_f64().unwrap_or(0.0)),
            };

            let mut resp = json_response(&economics, 200)?;
            resp.headers_mut().set("Cache-Control", "public, max-age=300")?;
            Ok(resp)
        })
        // Admin metrics
        .get_async("/api/admin/metrics", |_, ctx| async move {
            let d1 = ctx.env.d1("DB")?;

            let count_row = d1
                .prepare("SELECT COUNT(*) as cnt FROM orders")
                .first::<serde_json::Value>(None)
                .await?;
            let order_count = count_row.and_then(|v| v["cnt"].as_i64()).unwrap_or(0);

            let mut orders_by_status = std::collections::HashMap::new();
            let status_rows = d1
                .prepare("SELECT status, COUNT(*) as cnt FROM orders GROUP BY status")
                .all()
                .await?
                .results::<serde_json::Value>()?;
            for row in status_rows {
                if let (Some(s), Some(c)) = (row["status"].as_str(), row["cnt"].as_i64()) {
                    orders_by_status.insert(s.to_string(), c);
                }
            }

            let otd = d1
                .prepare(
                    "SELECT
                        SUM(CASE WHEN (julianday(actual_delivery_at) - julianday(created_at)) * 1440 < estimated_eta THEN 1 ELSE 0 END) as on_time,
                        COUNT(*) as total
                     FROM orders WHERE actual_delivery_at IS NOT NULL AND estimated_eta IS NOT NULL",
                )
                .first::<serde_json::Value>(None)
                .await?
                .unwrap_or(serde_json::json!({}));
            let on_time = otd["on_time"].as_i64().unwrap_or(0);
            let total_delivered = otd["total"].as_i64().unwrap_or(0);
            let on_time_delivery_rate = if total_delivered > 0 {
                on_time as f64 / total_delivered as f64 * 100.0
            } else {
                0.0
            };

            let eta_row = d1
                .prepare(
                    "SELECT COALESCE(AVG(ABS((julianday(actual_delivery_at) - julianday(created_at)) * 1440 - estimated_eta)), 0) as avg_err
                     FROM orders WHERE actual_delivery_at IS NOT NULL AND estimated_eta IS NOT NULL",
                )
                .first::<serde_json::Value>(None)
                .await?;
            let avg_eta_error_minutes = eta_row.and_then(|v| v["avg_err"].as_f64()).unwrap_or(0.0);

            let rev = d1
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
                .await?
                .unwrap_or(serde_json::json!({}));

            let cu = d1
                .prepare("SELECT SUM(CASE WHEN available=1 THEN 1 ELSE 0 END) as avail, COUNT(*) as total FROM couriers")
                .first::<serde_json::Value>(None)
                .await?
                .unwrap_or(serde_json::json!({}));

            let mut orders_by_zone = std::collections::HashMap::new();
            let zone_rows = d1
                .prepare("SELECT z.name, COUNT(o.id) as cnt FROM orders o JOIN zones z ON o.zone_id = z.id GROUP BY z.name")
                .all()
                .await?
                .results::<serde_json::Value>()?;
            for row in zone_rows {
                if let (Some(n), Some(c)) = (row["name"].as_str(), row["cnt"].as_i64()) {
                    orders_by_zone.insert(n.to_string(), c);
                }
            }

            json_response(
                &AdminMetrics {
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
                    courier_utilization: CourierUtil {
                        available: cu["avail"].as_i64().unwrap_or(0),
                        total: cu["total"].as_i64().unwrap_or(0),
                    },
                    orders_by_zone,
                },
                200,
            )
        })
        .run(req, env)
        .await
}

// ── D1 order loader ────────────────────────────────────────────────────

async fn load_order_d1(d1: &d1::D1Database, order_id: &str) -> Result<Option<OrderResp>> {
    let row = d1
        .prepare(
            "SELECT id, restaurant_id, courier_id, customer_address, zone_id, status,
                    food_total, delivery_fee, tip, federal_fee, local_ops_fee, processing_fee,
                    created_at, updated_at, estimated_eta, actual_delivery_at
             FROM orders WHERE id = ?1",
        )
        .bind(&[order_id.into()])?
        .first::<OrderRow>(None)
        .await?;

    let row = match row {
        Some(r) => r,
        None => return Ok(None),
    };

    let items = d1
        .prepare(
            "SELECT menu_item_id, name, quantity, unit_price FROM order_items WHERE order_id = ?1",
        )
        .bind(&[order_id.into()])?
        .all()
        .await?
        .results::<OrderItemRow>()?;

    Ok(Some(to_order_resp(row, items)))
}
