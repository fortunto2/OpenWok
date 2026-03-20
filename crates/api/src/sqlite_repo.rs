use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use openwok_core::money::Money;
use openwok_core::order::{Order, OrderItem, OrderStatus};
use openwok_core::pricing::PricingBreakdown;
use openwok_core::repo::{
    AdminMetrics, AssignCourierResult, CourierUtilization, CreateCourierRequest,
    CreateOrderRequest, CreateRestaurantRequest, PublicEconomics, RepoError, Repository,
    RevenueBreakdown,
};
use openwok_core::types::{
    Courier, CourierId, CourierKind, MenuItem, MenuItemId, OrderId, Restaurant, RestaurantId,
    ZoneId,
};
use rusqlite::params;
use tokio::sync::Mutex;

pub struct SqliteRepo {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

impl SqliteRepo {
    pub fn new(conn: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self { conn }
    }
}

fn row_to_restaurant(
    conn: &rusqlite::Connection,
    id: &str,
    name: String,
    zone_id: &str,
    active: bool,
) -> Restaurant {
    let mut stmt = conn
        .prepare("SELECT id, name, price FROM menu_items WHERE restaurant_id = ?1")
        .unwrap();
    let menu: Vec<MenuItem> = stmt
        .query_map(params![id], |row| {
            let mid: String = row.get(0)?;
            let mname: String = row.get(1)?;
            let mprice: String = row.get(2)?;
            Ok(MenuItem {
                id: MenuItemId::from_uuid(uuid::Uuid::parse_str(&mid).unwrap()),
                name: mname,
                price: Money::from(mprice.as_str()),
                restaurant_id: RestaurantId::from_uuid(uuid::Uuid::parse_str(id).unwrap()),
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    Restaurant {
        id: RestaurantId::from_uuid(uuid::Uuid::parse_str(id).unwrap()),
        name,
        zone_id: ZoneId::from_uuid(uuid::Uuid::parse_str(zone_id).unwrap()),
        menu,
        active,
    }
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
            .with_timezone(&chrono::Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
            .unwrap()
            .with_timezone(&chrono::Utc),
        estimated_eta,
        actual_delivery_at: actual_delivery_at.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        }),
    })
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

#[async_trait]
impl Repository for SqliteRepo {
    async fn list_restaurants(&self) -> Result<Vec<Restaurant>, RepoError> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT id, name, zone_id, active FROM restaurants WHERE active = 1")
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        let restaurants: Vec<Restaurant> = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let zone_id: String = row.get(2)?;
                let active: bool = row.get(3)?;
                Ok((id, name, zone_id, active))
            })
            .map_err(|e| RepoError::Internal(e.to_string()))?
            .filter_map(|r| r.ok())
            .map(|(id, name, zone_id, active)| {
                row_to_restaurant(&conn, &id, name, &zone_id, active)
            })
            .collect();
        Ok(restaurants)
    }

    async fn get_restaurant(&self, id: RestaurantId) -> Result<Restaurant, RepoError> {
        let conn = self.conn.lock().await;
        let id_str = id.to_string();
        let result = conn.query_row(
            "SELECT id, name, zone_id, active FROM restaurants WHERE id = ?1",
            params![id_str],
            |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let zone_id: String = row.get(2)?;
                let active: bool = row.get(3)?;
                Ok((id, name, zone_id, active))
            },
        );

        match result {
            Ok((id, name, zone_id, active)) => {
                Ok(row_to_restaurant(&conn, &id, name, &zone_id, active))
            }
            Err(_) => Err(RepoError::NotFound),
        }
    }

    async fn create_restaurant(
        &self,
        req: CreateRestaurantRequest,
    ) -> Result<Restaurant, RepoError> {
        let conn = self.conn.lock().await;
        let id = RestaurantId::new();
        let id_str = id.to_string();
        let zone_str = req.zone_id.to_string();

        conn.execute(
            "INSERT OR IGNORE INTO zones (id, name) VALUES (?1, ?2)",
            params![zone_str, format!("Zone {}", &zone_str[..8])],
        )
        .map_err(|e| RepoError::Internal(e.to_string()))?;

        conn.execute(
            "INSERT INTO restaurants (id, name, zone_id, active) VALUES (?1, ?2, ?3, 1)",
            params![id_str, req.name, zone_str],
        )
        .map_err(|e| RepoError::Internal(e.to_string()))?;

        let menu: Vec<MenuItem> = req
            .menu
            .into_iter()
            .map(|m| {
                let mid = MenuItemId::new();
                let mid_str = mid.to_string();
                let price_str = m.price.amount().to_string();
                conn.execute(
                    "INSERT INTO menu_items (id, restaurant_id, name, price) VALUES (?1, ?2, ?3, ?4)",
                    params![mid_str, id_str, m.name, price_str],
                )
                .unwrap();
                MenuItem {
                    id: mid,
                    name: m.name,
                    price: m.price,
                    restaurant_id: id,
                }
            })
            .collect();

        Ok(Restaurant {
            id,
            name: req.name,
            zone_id: req.zone_id,
            menu,
            active: true,
        })
    }

    async fn list_orders(&self) -> Result<Vec<Order>, RepoError> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT id FROM orders")
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        let ids: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .map_err(|e| RepoError::Internal(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        let orders: Vec<Order> = ids.iter().filter_map(|id| load_order(&conn, id)).collect();
        Ok(orders)
    }

    async fn get_order(&self, id: OrderId) -> Result<Order, RepoError> {
        let conn = self.conn.lock().await;
        load_order(&conn, &id.to_string()).ok_or(RepoError::NotFound)
    }

    async fn create_order(&self, req: CreateOrderRequest) -> Result<Order, RepoError> {
        let conn = self.conn.lock().await;

        // Validate restaurant exists
        let rest_id_str = req.restaurant_id.to_string();
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM restaurants WHERE id = ?1",
                params![rest_id_str],
                |r| r.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);

        if !exists {
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
        .map_err(|e| RepoError::Internal(e.to_string()))?;

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
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        }

        Ok(order)
    }

    async fn update_order_status(
        &self,
        id: OrderId,
        status: OrderStatus,
    ) -> Result<Order, RepoError> {
        let conn = self.conn.lock().await;
        let id_str = id.to_string();

        let mut order = load_order(&conn, &id_str).ok_or(RepoError::NotFound)?;

        order
            .transition(status)
            .map_err(|e| RepoError::InvalidTransition(e.to_string()))?;

        let updated = order.updated_at.to_rfc3339();
        conn.execute(
            "UPDATE orders SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![format!("{:?}", order.status), updated, id_str],
        )
        .map_err(|e| RepoError::Internal(e.to_string()))?;

        Ok(order)
    }

    async fn assign_courier(&self, order_id: OrderId) -> Result<AssignCourierResult, RepoError> {
        let conn = self.conn.lock().await;
        let order_id_str = order_id.to_string();

        let zone_id: String = conn
            .query_row(
                "SELECT zone_id FROM orders WHERE id = ?1",
                params![order_id_str],
                |r| r.get(0),
            )
            .map_err(|_| RepoError::NotFound)?;

        let courier_id: String = conn
            .query_row(
                "SELECT id FROM couriers WHERE available = 1 AND zone_id = ?1 LIMIT 1",
                params![zone_id],
                |r| r.get(0),
            )
            .map_err(|_| RepoError::Conflict("no available courier in zone".into()))?;

        conn.execute(
            "UPDATE couriers SET available = 0 WHERE id = ?1",
            params![courier_id],
        )
        .map_err(|e| RepoError::Internal(e.to_string()))?;

        conn.execute(
            "UPDATE orders SET courier_id = ?1 WHERE id = ?2",
            params![courier_id, order_id_str],
        )
        .map_err(|e| RepoError::Internal(e.to_string()))?;

        Ok(AssignCourierResult {
            order_id: order_id_str,
            courier_id,
        })
    }

    async fn list_couriers(&self) -> Result<Vec<Courier>, RepoError> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT id, name, kind, zone_id, available FROM couriers WHERE available = 1")
            .map_err(|e| RepoError::Internal(e.to_string()))?;
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
            .map_err(|e| RepoError::Internal(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(couriers)
    }

    async fn create_courier(&self, req: CreateCourierRequest) -> Result<Courier, RepoError> {
        let conn = self.conn.lock().await;
        let id = CourierId::new();
        let id_str = id.to_string();
        let zone_str = req.zone_id.to_string();

        conn.execute(
            "INSERT OR IGNORE INTO zones (id, name) VALUES (?1, ?2)",
            params![zone_str, format!("Zone {}", &zone_str[..8])],
        )
        .map_err(|e| RepoError::Internal(e.to_string()))?;

        conn.execute(
            "INSERT INTO couriers (id, name, kind, zone_id, available) VALUES (?1, ?2, ?3, ?4, 1)",
            params![id_str, req.name, "Human", zone_str],
        )
        .map_err(|e| RepoError::Internal(e.to_string()))?;

        Ok(Courier {
            id,
            name: req.name,
            kind: CourierKind::Human,
            zone_id: req.zone_id,
            available: true,
        })
    }

    async fn toggle_courier_available(
        &self,
        id: CourierId,
        available: bool,
    ) -> Result<Courier, RepoError> {
        let conn = self.conn.lock().await;
        let id_str = id.to_string();

        let updated = conn
            .execute(
                "UPDATE couriers SET available = ?1 WHERE id = ?2",
                params![available, id_str],
            )
            .map_err(|e| RepoError::Internal(e.to_string()))?;

        if updated == 0 {
            return Err(RepoError::NotFound);
        }

        conn.query_row(
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
        .map_err(|_| RepoError::NotFound)
    }

    async fn get_economics(&self) -> Result<PublicEconomics, RepoError> {
        let conn = self.conn.lock().await;
        conn.query_row(
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
            [],
            |row| {
                Ok(PublicEconomics {
                    total_orders: row.get(0)?,
                    total_food_revenue: format!("{:.2}", row.get::<_, f64>(1)?),
                    total_delivery_fees: format!("{:.2}", row.get::<_, f64>(2)?),
                    total_federal_fees: format!("{:.2}", row.get::<_, f64>(3)?),
                    total_local_ops_fees: format!("{:.2}", row.get::<_, f64>(4)?),
                    total_processing_fees: format!("{:.2}", row.get::<_, f64>(5)?),
                    avg_order_value: format!("{:.2}", row.get::<_, f64>(6)?),
                })
            },
        )
        .map_err(|e| RepoError::Internal(e.to_string()))
    }

    async fn get_metrics(&self) -> Result<AdminMetrics, RepoError> {
        let conn = self.conn.lock().await;

        let order_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM orders", [], |r| r.get(0))
            .unwrap_or(0);

        let mut orders_by_status = HashMap::new();
        {
            let mut stmt = conn
                .prepare("SELECT status, COUNT(*) FROM orders GROUP BY status")
                .map_err(|e| RepoError::Internal(e.to_string()))?;
            let rows = stmt
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
                .map_err(|e| RepoError::Internal(e.to_string()))?;
            for row in rows.flatten() {
                orders_by_status.insert(row.0, row.1);
            }
        }

        let (on_time, total_delivered) = conn
            .query_row(
                "SELECT
                    SUM(CASE WHEN (julianday(actual_delivery_at) - julianday(created_at)) * 1440 < estimated_eta THEN 1 ELSE 0 END),
                    COUNT(*)
                 FROM orders
                 WHERE actual_delivery_at IS NOT NULL AND estimated_eta IS NOT NULL",
                [],
                |r| Ok((r.get::<_, Option<i64>>(0)?, r.get::<_, i64>(1)?)),
            )
            .unwrap_or((None, 0));

        let on_time_delivery_rate = if total_delivered > 0 {
            on_time.unwrap_or(0) as f64 / total_delivered as f64 * 100.0
        } else {
            0.0
        };

        let avg_eta_error_minutes: f64 = conn
            .query_row(
                "SELECT COALESCE(AVG(ABS((julianday(actual_delivery_at) - julianday(created_at)) * 1440 - estimated_eta)), 0)
                 FROM orders
                 WHERE actual_delivery_at IS NOT NULL AND estimated_eta IS NOT NULL",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0.0);

        let revenue_breakdown = conn
            .query_row(
                "SELECT
                    COALESCE(SUM(CAST(food_total AS REAL)), 0),
                    COALESCE(SUM(CAST(delivery_fee AS REAL)), 0),
                    COALESCE(SUM(CAST(federal_fee AS REAL)), 0),
                    COALESCE(SUM(CAST(local_ops_fee AS REAL)), 0),
                    COALESCE(SUM(CAST(processing_fee AS REAL)), 0)
                 FROM orders",
                [],
                |r| {
                    Ok(RevenueBreakdown {
                        total_food_revenue: format!("{:.2}", r.get::<_, f64>(0)?),
                        total_delivery_fees: format!("{:.2}", r.get::<_, f64>(1)?),
                        total_federal_fees: format!("{:.2}", r.get::<_, f64>(2)?),
                        total_local_ops_fees: format!("{:.2}", r.get::<_, f64>(3)?),
                        total_processing_fees: format!("{:.2}", r.get::<_, f64>(4)?),
                    })
                },
            )
            .unwrap_or(RevenueBreakdown {
                total_food_revenue: "0.00".into(),
                total_delivery_fees: "0.00".into(),
                total_federal_fees: "0.00".into(),
                total_local_ops_fees: "0.00".into(),
                total_processing_fees: "0.00".into(),
            });

        let courier_utilization = conn
            .query_row(
                "SELECT
                    SUM(CASE WHEN available = 1 THEN 1 ELSE 0 END),
                    COUNT(*)
                 FROM couriers",
                [],
                |r| {
                    Ok(CourierUtilization {
                        available: r.get::<_, Option<i64>>(0)?.unwrap_or(0),
                        total: r.get(1)?,
                    })
                },
            )
            .unwrap_or(CourierUtilization {
                available: 0,
                total: 0,
            });

        let mut orders_by_zone = HashMap::new();
        {
            let mut stmt = conn
                .prepare(
                    "SELECT z.name, COUNT(o.id)
                     FROM orders o
                     JOIN zones z ON o.zone_id = z.id
                     GROUP BY z.name",
                )
                .map_err(|e| RepoError::Internal(e.to_string()))?;
            let rows = stmt
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
                .map_err(|e| RepoError::Internal(e.to_string()))?;
            for row in rows.flatten() {
                orders_by_zone.insert(row.0, row.1);
            }
        }

        Ok(AdminMetrics {
            order_count,
            orders_by_status,
            on_time_delivery_rate,
            avg_eta_error_minutes,
            revenue_breakdown,
            courier_utilization,
            orders_by_zone,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use openwok_core::order::OrderStatus;
    use openwok_core::repo::{
        CreateCourierRequest, CreateMenuItemRequest, CreateOrderItemRequest, CreateOrderRequest,
        CreateRestaurantRequest, Repository,
    };

    fn test_repo() -> SqliteRepo {
        let conn = db::open(":memory:");
        SqliteRepo::new(Arc::new(Mutex::new(conn)))
    }

    fn seeded_repo() -> SqliteRepo {
        let conn = db::open(":memory:");
        db::seed_la_data(&conn);
        SqliteRepo::new(Arc::new(Mutex::new(conn)))
    }

    #[tokio::test]
    async fn list_restaurants_returns_seeded() {
        let repo = seeded_repo();
        let restaurants = repo.list_restaurants().await.unwrap();
        assert_eq!(restaurants.len(), 18);
    }

    #[tokio::test]
    async fn get_restaurant_not_found() {
        let repo = test_repo();
        let result = repo.get_restaurant(RestaurantId::new()).await;
        assert!(matches!(result, Err(RepoError::NotFound)));
    }

    #[tokio::test]
    async fn create_restaurant_returns_with_menu() {
        let repo = test_repo();
        let req = CreateRestaurantRequest {
            name: "Test Wok".into(),
            zone_id: ZoneId::new(),
            menu: vec![CreateMenuItemRequest {
                name: "Pad Thai".into(),
                price: Money::from("12.99"),
            }],
        };
        let restaurant = repo.create_restaurant(req).await.unwrap();
        assert_eq!(restaurant.name, "Test Wok");
        assert_eq!(restaurant.menu.len(), 1);
        assert!(restaurant.active);
    }

    #[tokio::test]
    async fn create_order_returns_pricing_breakdown() {
        let repo = seeded_repo();
        let restaurants = repo.list_restaurants().await.unwrap();
        let rest = &restaurants[0];
        let item = &rest.menu[0];

        let req = CreateOrderRequest {
            restaurant_id: rest.id,
            items: vec![CreateOrderItemRequest {
                menu_item_id: item.id,
                name: item.name.clone(),
                quantity: 2,
                unit_price: item.price,
            }],
            customer_address: "123 Test St".into(),
            zone_id: rest.zone_id,
            delivery_fee: Money::from("5.00"),
            tip: Money::from("3.00"),
            local_ops_fee: Money::from("2.50"),
        };
        let order = repo.create_order(req).await.unwrap();

        assert_eq!(order.status, OrderStatus::Created);
        assert_eq!(order.pricing.federal_fee, Money::from("1.00"));
        assert_eq!(order.pricing.delivery_fee, Money::from("5.00"));
        assert_eq!(order.pricing.tip, Money::from("3.00"));
        assert_eq!(order.pricing.local_ops_fee, Money::from("2.50"));
    }

    #[tokio::test]
    async fn create_order_nonexistent_restaurant() {
        let repo = test_repo();
        let req = CreateOrderRequest {
            restaurant_id: RestaurantId::new(),
            items: vec![CreateOrderItemRequest {
                menu_item_id: MenuItemId::new(),
                name: "Item".into(),
                quantity: 1,
                unit_price: Money::from("10.00"),
            }],
            customer_address: "123 Test".into(),
            zone_id: ZoneId::new(),
            delivery_fee: Money::from("5.00"),
            tip: Money::from("0.00"),
            local_ops_fee: Money::from("2.00"),
        };
        let result = repo.create_order(req).await;
        assert!(matches!(result, Err(RepoError::NotFound)));
    }

    #[tokio::test]
    async fn update_order_status_valid_transition() {
        let repo = seeded_repo();
        let restaurants = repo.list_restaurants().await.unwrap();
        let rest = &restaurants[0];
        let item = &rest.menu[0];

        let order = repo
            .create_order(CreateOrderRequest {
                restaurant_id: rest.id,
                items: vec![CreateOrderItemRequest {
                    menu_item_id: item.id,
                    name: item.name.clone(),
                    quantity: 1,
                    unit_price: item.price,
                }],
                customer_address: "456 Oak".into(),
                zone_id: rest.zone_id,
                delivery_fee: Money::from("5.00"),
                tip: Money::from("0.00"),
                local_ops_fee: Money::from("2.00"),
            })
            .await
            .unwrap();

        let updated = repo
            .update_order_status(order.id, OrderStatus::Confirmed)
            .await
            .unwrap();
        assert_eq!(updated.status, OrderStatus::Confirmed);
    }

    #[tokio::test]
    async fn update_order_status_invalid_transition() {
        let repo = seeded_repo();
        let restaurants = repo.list_restaurants().await.unwrap();
        let rest = &restaurants[0];
        let item = &rest.menu[0];

        let order = repo
            .create_order(CreateOrderRequest {
                restaurant_id: rest.id,
                items: vec![CreateOrderItemRequest {
                    menu_item_id: item.id,
                    name: item.name.clone(),
                    quantity: 1,
                    unit_price: item.price,
                }],
                customer_address: "789 Pine".into(),
                zone_id: rest.zone_id,
                delivery_fee: Money::from("5.00"),
                tip: Money::from("0.00"),
                local_ops_fee: Money::from("2.00"),
            })
            .await
            .unwrap();

        let result = repo
            .update_order_status(order.id, OrderStatus::Delivered)
            .await;
        assert!(matches!(result, Err(RepoError::InvalidTransition(_))));
    }

    #[tokio::test]
    async fn get_economics_empty_db() {
        let repo = test_repo();
        let economics = repo.get_economics().await.unwrap();
        assert_eq!(economics.total_orders, 0);
        assert_eq!(economics.total_food_revenue, "0.00");
        assert_eq!(economics.avg_order_value, "0.00");
    }

    #[tokio::test]
    async fn get_economics_with_order() {
        let repo = seeded_repo();
        let restaurants = repo.list_restaurants().await.unwrap();
        let rest = &restaurants[0];
        let item = &rest.menu[0];

        repo.create_order(CreateOrderRequest {
            restaurant_id: rest.id,
            items: vec![CreateOrderItemRequest {
                menu_item_id: item.id,
                name: item.name.clone(),
                quantity: 2,
                unit_price: item.price,
            }],
            customer_address: "100 Main".into(),
            zone_id: rest.zone_id,
            delivery_fee: Money::from("5.00"),
            tip: Money::from("3.00"),
            local_ops_fee: Money::from("2.50"),
        })
        .await
        .unwrap();

        let economics = repo.get_economics().await.unwrap();
        assert_eq!(economics.total_orders, 1);
        assert_eq!(economics.total_federal_fees, "1.00");
    }

    #[tokio::test]
    async fn get_metrics_empty_db() {
        let repo = test_repo();
        let metrics = repo.get_metrics().await.unwrap();
        assert_eq!(metrics.order_count, 0);
        assert_eq!(metrics.on_time_delivery_rate, 0.0);
    }
}
