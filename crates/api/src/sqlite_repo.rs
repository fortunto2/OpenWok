use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use openwok_core::money::Money;
use openwok_core::order::{Order, OrderItem, OrderStatus};
use openwok_core::pricing::PricingBreakdown;
use openwok_core::repo::{
    AdminMetrics, AssignCourierResult, CourierUtilization, CreateCourierRequest,
    CreateMenuItemRequest, CreateOrderRequest, CreateRestaurantRequest, PublicEconomics, RepoError,
    Repository, RevenueBreakdown,
};
use openwok_core::types::{
    Courier, CourierId, CourierKind, CreatePaymentRequest, CreateUserRequest, MenuItem, MenuItemId,
    OrderId, Payment, PaymentId, PaymentStatus, Restaurant, RestaurantId, UpdateMenuItemRequest,
    UpdatePaymentStatusRequest, UpdateRestaurantRequest, User, UserId, UserRole, ZoneId,
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

#[allow(clippy::too_many_arguments)]
fn row_to_restaurant(
    conn: &rusqlite::Connection,
    id: &str,
    name: String,
    zone_id: &str,
    active: bool,
    owner_id: Option<String>,
    description: Option<String>,
    address: Option<String>,
    phone: Option<String>,
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
        owner_id: owner_id.map(|o| UserId::from_uuid(uuid::Uuid::parse_str(&o).unwrap())),
        description,
        address,
        phone,
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

fn row_to_courier(
    id: &str,
    name: String,
    kind: &str,
    zone_id: &str,
    available: bool,
    user_id: Option<String>,
) -> Courier {
    Courier {
        id: CourierId::from_uuid(uuid::Uuid::parse_str(id).unwrap()),
        name,
        kind: match kind {
            "Human" => CourierKind::Human,
            _ => CourierKind::Human,
        },
        zone_id: ZoneId::from_uuid(uuid::Uuid::parse_str(zone_id).unwrap()),
        available,
        user_id,
    }
}

#[async_trait]
impl Repository for SqliteRepo {
    async fn list_restaurants(&self) -> Result<Vec<Restaurant>, RepoError> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT id, name, zone_id, active, owner_id, description, address, phone FROM restaurants WHERE active = 1")
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        let restaurants: Vec<Restaurant> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, bool>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                ))
            })
            .map_err(|e| RepoError::Internal(e.to_string()))?
            .filter_map(|r| r.ok())
            .map(
                |(id, name, zone_id, active, owner_id, description, address, phone)| {
                    row_to_restaurant(
                        &conn,
                        &id,
                        name,
                        &zone_id,
                        active,
                        owner_id,
                        description,
                        address,
                        phone,
                    )
                },
            )
            .collect();
        Ok(restaurants)
    }

    async fn get_restaurant(&self, id: RestaurantId) -> Result<Restaurant, RepoError> {
        let conn = self.conn.lock().await;
        let id_str = id.to_string();
        let result = conn.query_row(
            "SELECT id, name, zone_id, active, owner_id, description, address, phone FROM restaurants WHERE id = ?1",
            params![id_str],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, bool>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                ))
            },
        );

        match result {
            Ok((id, name, zone_id, active, owner_id, description, address, phone)) => {
                Ok(row_to_restaurant(
                    &conn,
                    &id,
                    name,
                    &zone_id,
                    active,
                    owner_id,
                    description,
                    address,
                    phone,
                ))
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
        let now = chrono::Utc::now().to_rfc3339();
        let owner_id_str = req.owner_id.map(|o| o.to_string());

        conn.execute(
            "INSERT OR IGNORE INTO zones (id, name) VALUES (?1, ?2)",
            params![zone_str, format!("Zone {}", &zone_str[..8])],
        )
        .map_err(|e| RepoError::Internal(e.to_string()))?;

        conn.execute(
            "INSERT INTO restaurants (id, name, zone_id, active, owner_id, description, address, phone, created_at, updated_at) VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![id_str, req.name, zone_str, owner_id_str, req.description, req.address, req.phone, now, now],
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
            owner_id: req.owner_id,
            description: req.description,
            address: req.address,
            phone: req.phone,
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
            .prepare("SELECT id, name, kind, zone_id, available, user_id FROM couriers WHERE available = 1")
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        let couriers: Vec<Courier> = stmt
            .query_map([], |row| {
                Ok(row_to_courier(
                    &row.get::<_, String>(0)?,
                    row.get(1)?,
                    &row.get::<_, String>(2)?,
                    &row.get::<_, String>(3)?,
                    row.get(4)?,
                    row.get(5)?,
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
            "INSERT INTO couriers (id, name, kind, zone_id, available, user_id) VALUES (?1, ?2, ?3, ?4, 1, ?5)",
            params![id_str, req.name, "Human", zone_str, req.user_id],
        )
        .map_err(|e| RepoError::Internal(e.to_string()))?;

        Ok(Courier {
            id,
            name: req.name,
            kind: CourierKind::Human,
            zone_id: req.zone_id,
            available: true,
            user_id: req.user_id,
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
            "SELECT id, name, kind, zone_id, available, user_id FROM couriers WHERE id = ?1",
            params![id_str],
            |row| {
                Ok(row_to_courier(
                    &row.get::<_, String>(0)?,
                    row.get(1)?,
                    &row.get::<_, String>(2)?,
                    &row.get::<_, String>(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .map_err(|_| RepoError::NotFound)
    }

    async fn create_user(&self, req: CreateUserRequest) -> Result<User, RepoError> {
        let conn = self.conn.lock().await;
        let id = UserId::new();
        let id_str = id.to_string();
        let role = req.role.unwrap_or(UserRole::Customer);
        let now = chrono::Utc::now();
        let created_at = now.to_rfc3339();

        conn.execute(
            "INSERT INTO users (id, supabase_user_id, email, name, role, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id_str, req.supabase_user_id, req.email, req.name, role.to_string(), created_at],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                RepoError::Conflict("user with this supabase_user_id already exists".into())
            } else {
                RepoError::Internal(e.to_string())
            }
        })?;

        Ok(User {
            id,
            supabase_user_id: req.supabase_user_id,
            email: req.email,
            name: req.name,
            role,
            blocked: false,
            created_at: now,
        })
    }

    async fn get_user(&self, id: UserId) -> Result<User, RepoError> {
        let conn = self.conn.lock().await;
        conn.query_row(
            "SELECT id, supabase_user_id, email, name, role, created_at, blocked FROM users WHERE id = ?1",
            params![id.to_string()],
            |row| {
                Ok(User {
                    id: UserId::from_uuid(
                        uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    ),
                    supabase_user_id: row.get(1)?,
                    email: row.get(2)?,
                    name: row.get(3)?,
                    role: row
                        .get::<_, String>(4)?
                        .parse::<UserRole>()
                        .unwrap_or(UserRole::Customer),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    blocked: row.get::<_, i32>(6).unwrap_or(0) != 0,
                })
            },
        )
        .map_err(|_| RepoError::NotFound)
    }

    async fn get_user_by_supabase_id(&self, supabase_user_id: &str) -> Result<User, RepoError> {
        let conn = self.conn.lock().await;
        conn.query_row(
            "SELECT id, supabase_user_id, email, name, role, created_at, blocked FROM users WHERE supabase_user_id = ?1",
            params![supabase_user_id],
            |row| {
                Ok(User {
                    id: UserId::from_uuid(uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap()),
                    supabase_user_id: row.get(1)?,
                    email: row.get(2)?,
                    name: row.get(3)?,
                    role: row.get::<_, String>(4)?.parse::<UserRole>().unwrap_or(UserRole::Customer),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    blocked: row.get::<_, i32>(6).unwrap_or(0) != 0,
                })
            },
        )
        .map_err(|_| RepoError::NotFound)
    }

    async fn create_payment(&self, req: CreatePaymentRequest) -> Result<Payment, RepoError> {
        let conn = self.conn.lock().await;
        let id = PaymentId::new();
        let id_str = id.to_string();
        let now = chrono::Utc::now();
        let created_at = now.to_rfc3339();

        conn.execute(
            "INSERT INTO payments (id, order_id, stripe_checkout_session_id, status, amount_total, restaurant_amount, courier_amount, federal_amount, local_ops_amount, processing_amount, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                id_str,
                req.order_id.to_string(),
                req.stripe_checkout_session_id,
                "Pending",
                req.amount_total.amount().to_string(),
                req.restaurant_amount.amount().to_string(),
                req.courier_amount.amount().to_string(),
                req.federal_amount.amount().to_string(),
                req.local_ops_amount.amount().to_string(),
                req.processing_amount.amount().to_string(),
                created_at,
            ],
        )
        .map_err(|e| RepoError::Internal(e.to_string()))?;

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
            created_at: now,
        })
    }

    async fn get_payment_by_order(&self, order_id: OrderId) -> Result<Payment, RepoError> {
        let conn = self.conn.lock().await;
        conn.query_row(
            "SELECT id, order_id, stripe_payment_intent_id, stripe_checkout_session_id, status, amount_total, restaurant_amount, courier_amount, federal_amount, local_ops_amount, processing_amount, created_at
             FROM payments WHERE order_id = ?1",
            params![order_id.to_string()],
            |row| {
                Ok(Payment {
                    id: PaymentId::from_uuid(uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap()),
                    order_id: OrderId::from_uuid(uuid::Uuid::parse_str(&row.get::<_, String>(1)?).unwrap()),
                    stripe_payment_intent_id: row.get(2)?,
                    stripe_checkout_session_id: row.get(3)?,
                    status: row.get::<_, String>(4)?.parse::<PaymentStatus>().unwrap_or(PaymentStatus::Pending),
                    amount_total: Money::from(row.get::<_, String>(5)?.as_str()),
                    restaurant_amount: Money::from(row.get::<_, String>(6)?.as_str()),
                    courier_amount: Money::from(row.get::<_, String>(7)?.as_str()),
                    federal_amount: Money::from(row.get::<_, String>(8)?.as_str()),
                    local_ops_amount: Money::from(row.get::<_, String>(9)?.as_str()),
                    processing_amount: Money::from(row.get::<_, String>(10)?.as_str()),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(11)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                })
            },
        )
        .map_err(|_| RepoError::NotFound)
    }

    async fn update_payment_status(
        &self,
        id: PaymentId,
        req: UpdatePaymentStatusRequest,
    ) -> Result<Payment, RepoError> {
        let conn = self.conn.lock().await;
        let id_str = id.to_string();

        let updated = if let Some(ref pi_id) = req.stripe_payment_intent_id {
            conn.execute(
                "UPDATE payments SET status = ?1, stripe_payment_intent_id = ?2 WHERE id = ?3",
                params![req.status.to_string(), pi_id, id_str],
            )
        } else {
            conn.execute(
                "UPDATE payments SET status = ?1 WHERE id = ?2",
                params![req.status.to_string(), id_str],
            )
        }
        .map_err(|e| RepoError::Internal(e.to_string()))?;

        if updated == 0 {
            return Err(RepoError::NotFound);
        }

        conn.query_row(
            "SELECT id, order_id, stripe_payment_intent_id, stripe_checkout_session_id, status, amount_total, restaurant_amount, courier_amount, federal_amount, local_ops_amount, processing_amount, created_at
             FROM payments WHERE id = ?1",
            params![id_str],
            |row| {
                Ok(Payment {
                    id: PaymentId::from_uuid(uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap()),
                    order_id: OrderId::from_uuid(uuid::Uuid::parse_str(&row.get::<_, String>(1)?).unwrap()),
                    stripe_payment_intent_id: row.get(2)?,
                    stripe_checkout_session_id: row.get(3)?,
                    status: row.get::<_, String>(4)?.parse::<PaymentStatus>().unwrap_or(PaymentStatus::Pending),
                    amount_total: Money::from(row.get::<_, String>(5)?.as_str()),
                    restaurant_amount: Money::from(row.get::<_, String>(6)?.as_str()),
                    courier_amount: Money::from(row.get::<_, String>(7)?.as_str()),
                    federal_amount: Money::from(row.get::<_, String>(8)?.as_str()),
                    local_ops_amount: Money::from(row.get::<_, String>(9)?.as_str()),
                    processing_amount: Money::from(row.get::<_, String>(10)?.as_str()),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(11)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                })
            },
        )
        .map_err(|_| RepoError::NotFound)
    }

    async fn update_restaurant(
        &self,
        id: RestaurantId,
        req: UpdateRestaurantRequest,
    ) -> Result<Restaurant, RepoError> {
        {
            let conn = self.conn.lock().await;
            let id_str = id.to_string();
            let now = chrono::Utc::now().to_rfc3339();

            let mut sets = vec!["updated_at = ?".to_string()];
            let mut values: Vec<String> = vec![now];

            if let Some(ref name) = req.name {
                sets.push("name = ?".to_string());
                values.push(name.clone());
            }
            if let Some(ref desc) = req.description {
                sets.push("description = ?".to_string());
                values.push(desc.clone());
            }
            if let Some(ref addr) = req.address {
                sets.push("address = ?".to_string());
                values.push(addr.clone());
            }
            if let Some(ref phone) = req.phone {
                sets.push("phone = ?".to_string());
                values.push(phone.clone());
            }

            values.push(id_str);
            let sql = format!("UPDATE restaurants SET {} WHERE id = ?", sets.join(", "));
            let params_vec: Vec<&dyn rusqlite::types::ToSql> = values
                .iter()
                .map(|v| v as &dyn rusqlite::types::ToSql)
                .collect();
            let updated = conn
                .execute(&sql, params_vec.as_slice())
                .map_err(|e| RepoError::Internal(e.to_string()))?;

            if updated == 0 {
                return Err(RepoError::NotFound);
            }
        }
        self.get_restaurant(id).await
    }

    async fn toggle_restaurant_active(
        &self,
        id: RestaurantId,
        active: bool,
    ) -> Result<Restaurant, RepoError> {
        {
            let conn = self.conn.lock().await;
            let id_str = id.to_string();
            let now = chrono::Utc::now().to_rfc3339();

            let updated = conn
                .execute(
                    "UPDATE restaurants SET active = ?1, updated_at = ?2 WHERE id = ?3",
                    params![active, now, id_str],
                )
                .map_err(|e| RepoError::Internal(e.to_string()))?;

            if updated == 0 {
                return Err(RepoError::NotFound);
            }
        }
        self.get_restaurant(id).await
    }

    async fn list_restaurants_by_owner(
        &self,
        user_id: UserId,
    ) -> Result<Vec<Restaurant>, RepoError> {
        let conn = self.conn.lock().await;
        let user_id_str = user_id.to_string();
        let mut stmt = conn
            .prepare("SELECT id, name, zone_id, active, owner_id, description, address, phone FROM restaurants WHERE owner_id = ?1")
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        let restaurants: Vec<Restaurant> = stmt
            .query_map(params![user_id_str], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, bool>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                ))
            })
            .map_err(|e| RepoError::Internal(e.to_string()))?
            .filter_map(|r| r.ok())
            .map(|(id, name, zone_id, active, owner_id, desc, addr, phone)| {
                row_to_restaurant(
                    &conn, &id, name, &zone_id, active, owner_id, desc, addr, phone,
                )
            })
            .collect();
        Ok(restaurants)
    }

    async fn add_menu_item(
        &self,
        restaurant_id: RestaurantId,
        req: CreateMenuItemRequest,
    ) -> Result<MenuItem, RepoError> {
        let conn = self.conn.lock().await;
        let rid_str = restaurant_id.to_string();

        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM restaurants WHERE id = ?1",
                params![rid_str],
                |r| r.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);
        if !exists {
            return Err(RepoError::NotFound);
        }

        let mid = MenuItemId::new();
        let mid_str = mid.to_string();
        let price_str = req.price.amount().to_string();

        conn.execute(
            "INSERT INTO menu_items (id, restaurant_id, name, price) VALUES (?1, ?2, ?3, ?4)",
            params![mid_str, rid_str, req.name, price_str],
        )
        .map_err(|e| RepoError::Internal(e.to_string()))?;

        Ok(MenuItem {
            id: mid,
            name: req.name,
            price: req.price,
            restaurant_id,
        })
    }

    async fn get_menu_item(&self, id: MenuItemId) -> Result<MenuItem, RepoError> {
        let conn = self.conn.lock().await;
        let id_str = id.to_string();
        conn.query_row(
            "SELECT id, name, price, restaurant_id FROM menu_items WHERE id = ?1",
            params![id_str],
            |row| {
                let price_str: String = row.get(2)?;
                let rid: String = row.get(3)?;
                Ok(MenuItem {
                    id: MenuItemId::from_uuid(row.get::<_, String>(0)?.parse().unwrap_or_default()),
                    name: row.get(1)?,
                    price: Money::from(price_str.as_str()),
                    restaurant_id: RestaurantId::from_uuid(rid.parse().unwrap_or_default()),
                })
            },
        )
        .map_err(|_| RepoError::NotFound)
    }

    async fn update_menu_item(
        &self,
        id: MenuItemId,
        req: UpdateMenuItemRequest,
    ) -> Result<MenuItem, RepoError> {
        let conn = self.conn.lock().await;
        let id_str = id.to_string();

        let mut sets = Vec::new();
        let mut values: Vec<String> = Vec::new();

        if let Some(ref name) = req.name {
            sets.push("name = ?".to_string());
            values.push(name.clone());
        }
        if let Some(ref price) = req.price {
            sets.push("price = ?".to_string());
            values.push(price.amount().to_string());
        }

        if !sets.is_empty() {
            values.push(id_str.clone());
            let sql = format!("UPDATE menu_items SET {} WHERE id = ?", sets.join(", "));
            let params_vec: Vec<&dyn rusqlite::types::ToSql> = values
                .iter()
                .map(|v| v as &dyn rusqlite::types::ToSql)
                .collect();
            let updated = conn
                .execute(&sql, params_vec.as_slice())
                .map_err(|e| RepoError::Internal(e.to_string()))?;
            if updated == 0 {
                return Err(RepoError::NotFound);
            }
        }

        conn.query_row(
            "SELECT id, restaurant_id, name, price FROM menu_items WHERE id = ?1",
            params![id_str],
            |row| {
                Ok(MenuItem {
                    id: MenuItemId::from_uuid(
                        uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    ),
                    restaurant_id: RestaurantId::from_uuid(
                        uuid::Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                    ),
                    name: row.get(2)?,
                    price: Money::from(row.get::<_, String>(3)?.as_str()),
                })
            },
        )
        .map_err(|_| RepoError::NotFound)
    }

    async fn delete_menu_item(&self, id: MenuItemId) -> Result<(), RepoError> {
        let conn = self.conn.lock().await;
        let id_str = id.to_string();
        let deleted = conn
            .execute("DELETE FROM menu_items WHERE id = ?1", params![id_str])
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        if deleted == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn update_user_role(&self, user_id: UserId, role: UserRole) -> Result<User, RepoError> {
        let conn = self.conn.lock().await;
        let id_str = user_id.to_string();
        let updated = conn
            .execute(
                "UPDATE users SET role = ?1 WHERE id = ?2",
                params![role.to_string(), id_str],
            )
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        if updated == 0 {
            return Err(RepoError::NotFound);
        }

        conn.query_row(
            "SELECT id, supabase_user_id, email, name, role, created_at, blocked FROM users WHERE id = ?1",
            params![id_str],
            |row| {
                Ok(User {
                    id: UserId::from_uuid(
                        uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    ),
                    supabase_user_id: row.get(1)?,
                    email: row.get(2)?,
                    name: row.get(3)?,
                    role: row
                        .get::<_, String>(4)?
                        .parse::<UserRole>()
                        .unwrap_or(UserRole::Customer),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    blocked: row.get::<_, i32>(6).unwrap_or(0) != 0,
                })
            },
        )
        .map_err(|_| RepoError::NotFound)
    }

    async fn get_courier_by_user_id(&self, user_id: &str) -> Result<Courier, RepoError> {
        let conn = self.conn.lock().await;
        conn.query_row(
            "SELECT id, name, kind, zone_id, available, user_id FROM couriers WHERE user_id = ?1",
            params![user_id],
            |row| {
                Ok(row_to_courier(
                    &row.get::<_, String>(0)?,
                    row.get(1)?,
                    &row.get::<_, String>(2)?,
                    &row.get::<_, String>(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .map_err(|_| RepoError::NotFound)
    }

    async fn list_courier_orders(&self, courier_id: CourierId) -> Result<Vec<Order>, RepoError> {
        let conn = self.conn.lock().await;
        let cid = courier_id.to_string();
        let mut stmt = conn
            .prepare("SELECT id FROM orders WHERE courier_id = ?1 ORDER BY created_at DESC")
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        let order_ids: Vec<String> = stmt
            .query_map(params![cid], |row| row.get(0))
            .map_err(|e| RepoError::Internal(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        let orders: Vec<Order> = order_ids
            .iter()
            .filter_map(|id| load_order(&conn, id))
            .collect();
        Ok(orders)
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

    // ── Admin: users + disputes ──────────────────────────────────────

    async fn list_users(&self) -> Result<Vec<User>, RepoError> {
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT id, supabase_user_id, email, name, role, created_at, blocked FROM users ORDER BY created_at DESC")
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        let users = stmt
            .query_map([], |row| {
                Ok(User {
                    id: UserId::from_uuid(
                        uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    ),
                    supabase_user_id: row.get(1)?,
                    email: row.get(2)?,
                    name: row.get(3)?,
                    role: row
                        .get::<_, String>(4)?
                        .parse::<UserRole>()
                        .unwrap_or(UserRole::Customer),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    blocked: row.get::<_, i32>(6).unwrap_or(0) != 0,
                })
            })
            .map_err(|e| RepoError::Internal(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(users)
    }

    async fn set_user_blocked(&self, user_id: UserId, blocked: bool) -> Result<User, RepoError> {
        let conn = self.conn.lock().await;
        let id_str = user_id.to_string();
        let blocked_int: i32 = if blocked { 1 } else { 0 };
        let updated = conn
            .execute(
                "UPDATE users SET blocked = ?1 WHERE id = ?2",
                params![blocked_int, id_str],
            )
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        if updated == 0 {
            return Err(RepoError::NotFound);
        }
        conn.query_row(
            "SELECT id, supabase_user_id, email, name, role, created_at, blocked FROM users WHERE id = ?1",
            params![id_str],
            |row| {
                Ok(User {
                    id: UserId::from_uuid(uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap()),
                    supabase_user_id: row.get(1)?,
                    email: row.get(2)?,
                    name: row.get(3)?,
                    role: row.get::<_, String>(4)?.parse::<UserRole>().unwrap_or(UserRole::Customer),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    blocked: row.get::<_, i32>(6).unwrap_or(0) != 0,
                })
            },
        )
        .map_err(|_| RepoError::NotFound)
    }

    async fn create_dispute(
        &self,
        order_id: OrderId,
        user_id: UserId,
        reason: String,
    ) -> Result<openwok_core::types::Dispute, RepoError> {
        use openwok_core::types::{Dispute, DisputeId, DisputeStatus};
        let conn = self.conn.lock().await;
        let id = DisputeId::new();
        let now = chrono::Utc::now();
        let now_str = now.to_rfc3339();
        conn.execute(
            "INSERT INTO disputes (id, order_id, user_id, reason, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id.to_string(), order_id.to_string(), user_id.to_string(), reason, "Open", now_str],
        )
        .map_err(|e| RepoError::Internal(e.to_string()))?;
        Ok(Dispute {
            id,
            order_id,
            user_id,
            reason,
            status: DisputeStatus::Open,
            resolution: None,
            created_at: now,
            resolved_at: None,
        })
    }

    async fn list_disputes(&self) -> Result<Vec<openwok_core::types::Dispute>, RepoError> {
        use openwok_core::types::{Dispute, DisputeId, DisputeStatus};
        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT id, order_id, user_id, reason, status, resolution, created_at, resolved_at FROM disputes ORDER BY created_at DESC")
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        let disputes = stmt
            .query_map([], |row| {
                Ok(Dispute {
                    id: DisputeId::from_uuid(
                        uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    ),
                    order_id: OrderId::from_uuid(
                        uuid::Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                    ),
                    user_id: UserId::from_uuid(
                        uuid::Uuid::parse_str(&row.get::<_, String>(2)?).unwrap(),
                    ),
                    reason: row.get(3)?,
                    status: row
                        .get::<_, String>(4)?
                        .parse::<DisputeStatus>()
                        .unwrap_or(DisputeStatus::Open),
                    resolution: row.get(5)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    resolved_at: row.get::<_, Option<String>>(7)?.and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .ok()
                            .map(|d| d.with_timezone(&chrono::Utc))
                    }),
                })
            })
            .map_err(|e| RepoError::Internal(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(disputes)
    }

    async fn resolve_dispute(
        &self,
        id: openwok_core::types::DisputeId,
        status: openwok_core::types::DisputeStatus,
        resolution: Option<String>,
    ) -> Result<openwok_core::types::Dispute, RepoError> {
        use openwok_core::types::{Dispute, DisputeId, DisputeStatus};
        let conn = self.conn.lock().await;
        let id_str = id.to_string();
        let now = chrono::Utc::now();
        let now_str = now.to_rfc3339();
        let status_str = status.to_string();
        let updated = conn
            .execute(
                "UPDATE disputes SET status = ?1, resolution = ?2, resolved_at = ?3 WHERE id = ?4",
                params![status_str, resolution, now_str, id_str],
            )
            .map_err(|e| RepoError::Internal(e.to_string()))?;
        if updated == 0 {
            return Err(RepoError::NotFound);
        }
        conn.query_row(
            "SELECT id, order_id, user_id, reason, status, resolution, created_at, resolved_at FROM disputes WHERE id = ?1",
            params![id_str],
            |row| {
                Ok(Dispute {
                    id: DisputeId::from_uuid(uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap()),
                    order_id: OrderId::from_uuid(uuid::Uuid::parse_str(&row.get::<_, String>(1)?).unwrap()),
                    user_id: UserId::from_uuid(uuid::Uuid::parse_str(&row.get::<_, String>(2)?).unwrap()),
                    reason: row.get(3)?,
                    status: row.get::<_, String>(4)?.parse::<DisputeStatus>().unwrap_or(DisputeStatus::Open),
                    resolution: row.get(5)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    resolved_at: row.get::<_, Option<String>>(7)?.and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&chrono::Utc))
                    }),
                })
            },
        )
        .map_err(|_| RepoError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use openwok_core::order::OrderStatus;
    use openwok_core::repo::{
        CreateMenuItemRequest, CreateOrderItemRequest, CreateOrderRequest, CreateRestaurantRequest,
        Repository,
    };
    use openwok_core::types::{
        CreatePaymentRequest, CreateUserRequest, PaymentStatus, UpdatePaymentStatusRequest,
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
            owner_id: None,
            description: None,
            address: None,
            phone: None,
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

    #[tokio::test]
    async fn create_and_get_user() {
        let repo = test_repo();
        let user = repo
            .create_user(CreateUserRequest {
                supabase_user_id: "sub_abc123".into(),
                email: "test@example.com".into(),
                name: Some("Test User".into()),
                role: None,
            })
            .await
            .unwrap();

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, UserRole::Customer);

        let fetched = repo.get_user(user.id).await.unwrap();
        assert_eq!(fetched.email, "test@example.com");
        assert_eq!(fetched.supabase_user_id, "sub_abc123");
    }

    #[tokio::test]
    async fn get_user_by_supabase_id() {
        let repo = test_repo();
        let user = repo
            .create_user(CreateUserRequest {
                supabase_user_id: "sub_xyz".into(),
                email: "user@test.com".into(),
                name: None,
                role: Some(UserRole::RestaurantOwner),
            })
            .await
            .unwrap();

        let fetched = repo.get_user_by_supabase_id("sub_xyz").await.unwrap();
        assert_eq!(fetched.id, user.id);
        assert_eq!(fetched.role, UserRole::RestaurantOwner);
    }

    #[tokio::test]
    async fn create_user_duplicate_supabase_id() {
        let repo = test_repo();
        repo.create_user(CreateUserRequest {
            supabase_user_id: "dup_id".into(),
            email: "a@test.com".into(),
            name: None,
            role: None,
        })
        .await
        .unwrap();

        let result = repo
            .create_user(CreateUserRequest {
                supabase_user_id: "dup_id".into(),
                email: "b@test.com".into(),
                name: None,
                role: None,
            })
            .await;
        assert!(matches!(result, Err(RepoError::Conflict(_))));
    }

    #[tokio::test]
    async fn get_user_not_found() {
        let repo = test_repo();
        let result = repo.get_user(UserId::new()).await;
        assert!(matches!(result, Err(RepoError::NotFound)));
    }

    #[tokio::test]
    async fn create_and_get_payment() {
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
                customer_address: "123 Pay St".into(),
                zone_id: rest.zone_id,
                delivery_fee: Money::from("5.00"),
                tip: Money::from("2.00"),
                local_ops_fee: Money::from("2.50"),
            })
            .await
            .unwrap();

        let payment = repo
            .create_payment(CreatePaymentRequest {
                order_id: order.id,
                stripe_checkout_session_id: Some("cs_test_123".into()),
                amount_total: Money::from("30.00"),
                restaurant_amount: Money::from("20.00"),
                courier_amount: Money::from("5.00"),
                federal_amount: Money::from("1.00"),
                local_ops_amount: Money::from("2.50"),
                processing_amount: Money::from("1.50"),
            })
            .await
            .unwrap();

        assert_eq!(payment.status, PaymentStatus::Pending);
        assert_eq!(payment.amount_total, Money::from("30.00"));

        let fetched = repo.get_payment_by_order(order.id).await.unwrap();
        assert_eq!(fetched.id, payment.id);
        assert_eq!(
            fetched.stripe_checkout_session_id,
            Some("cs_test_123".into())
        );
    }

    #[tokio::test]
    async fn update_payment_status_to_succeeded() {
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
                customer_address: "456 Pay Ave".into(),
                zone_id: rest.zone_id,
                delivery_fee: Money::from("5.00"),
                tip: Money::from("0.00"),
                local_ops_fee: Money::from("2.00"),
            })
            .await
            .unwrap();

        let payment = repo
            .create_payment(CreatePaymentRequest {
                order_id: order.id,
                stripe_checkout_session_id: Some("cs_test_456".into()),
                amount_total: Money::from("25.00"),
                restaurant_amount: Money::from("15.00"),
                courier_amount: Money::from("5.00"),
                federal_amount: Money::from("1.00"),
                local_ops_amount: Money::from("2.00"),
                processing_amount: Money::from("2.00"),
            })
            .await
            .unwrap();

        let updated = repo
            .update_payment_status(
                payment.id,
                UpdatePaymentStatusRequest {
                    status: PaymentStatus::Succeeded,
                    stripe_payment_intent_id: Some("pi_test_789".into()),
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.status, PaymentStatus::Succeeded);
        assert_eq!(updated.stripe_payment_intent_id, Some("pi_test_789".into()));
    }

    #[tokio::test]
    async fn get_payment_not_found() {
        let repo = test_repo();
        let result = repo.get_payment_by_order(OrderId::new()).await;
        assert!(matches!(result, Err(RepoError::NotFound)));
    }

    // --- Restaurant management tests ---

    #[tokio::test]
    async fn create_restaurant_with_owner() {
        let repo = test_repo();
        let user = repo
            .create_user(CreateUserRequest {
                supabase_user_id: "owner_sub".into(),
                email: "owner@test.com".into(),
                name: Some("Owner".into()),
                role: None,
            })
            .await
            .unwrap();

        let req = CreateRestaurantRequest {
            name: "My Wok".into(),
            zone_id: ZoneId::new(),
            menu: vec![],
            owner_id: Some(user.id),
            description: Some("Great food".into()),
            address: Some("123 Main St".into()),
            phone: Some("555-1234".into()),
        };
        let r = repo.create_restaurant(req).await.unwrap();
        assert_eq!(r.owner_id, Some(user.id));
        assert_eq!(r.description, Some("Great food".into()));
        assert_eq!(r.address, Some("123 Main St".into()));
        assert_eq!(r.phone, Some("555-1234".into()));
    }

    #[tokio::test]
    async fn update_restaurant_changes_name() {
        let repo = test_repo();
        let r = repo
            .create_restaurant(CreateRestaurantRequest {
                name: "Old Name".into(),
                zone_id: ZoneId::new(),
                menu: vec![],
                owner_id: None,
                description: None,
                address: None,
                phone: None,
            })
            .await
            .unwrap();

        let updated = repo
            .update_restaurant(
                r.id,
                openwok_core::types::UpdateRestaurantRequest {
                    name: Some("New Name".into()),
                    description: Some("Desc".into()),
                    address: None,
                    phone: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.description, Some("Desc".into()));
    }

    #[tokio::test]
    async fn toggle_restaurant_active() {
        let repo = test_repo();
        let r = repo
            .create_restaurant(CreateRestaurantRequest {
                name: "Toggle Me".into(),
                zone_id: ZoneId::new(),
                menu: vec![],
                owner_id: None,
                description: None,
                address: None,
                phone: None,
            })
            .await
            .unwrap();
        assert!(r.active);

        let toggled = repo.toggle_restaurant_active(r.id, false).await.unwrap();
        assert!(!toggled.active);

        let toggled_back = repo.toggle_restaurant_active(r.id, true).await.unwrap();
        assert!(toggled_back.active);
    }

    #[tokio::test]
    async fn list_restaurants_by_owner() {
        let repo = test_repo();
        let user = repo
            .create_user(CreateUserRequest {
                supabase_user_id: "owner2".into(),
                email: "own2@test.com".into(),
                name: None,
                role: None,
            })
            .await
            .unwrap();

        // Create 2 restaurants for this owner
        for name in ["R1", "R2"] {
            repo.create_restaurant(CreateRestaurantRequest {
                name: name.into(),
                zone_id: ZoneId::new(),
                menu: vec![],
                owner_id: Some(user.id),
                description: None,
                address: None,
                phone: None,
            })
            .await
            .unwrap();
        }

        // Create 1 without owner
        repo.create_restaurant(CreateRestaurantRequest {
            name: "No Owner".into(),
            zone_id: ZoneId::new(),
            menu: vec![],
            owner_id: None,
            description: None,
            address: None,
            phone: None,
        })
        .await
        .unwrap();

        let owned = repo.list_restaurants_by_owner(user.id).await.unwrap();
        assert_eq!(owned.len(), 2);
    }

    #[tokio::test]
    async fn add_and_update_and_delete_menu_item() {
        let repo = test_repo();
        let r = repo
            .create_restaurant(CreateRestaurantRequest {
                name: "Menu Test".into(),
                zone_id: ZoneId::new(),
                menu: vec![],
                owner_id: None,
                description: None,
                address: None,
                phone: None,
            })
            .await
            .unwrap();

        // Add
        let item = repo
            .add_menu_item(
                r.id,
                CreateMenuItemRequest {
                    name: "Pad Thai".into(),
                    price: Money::from("12.99"),
                },
            )
            .await
            .unwrap();
        assert_eq!(item.name, "Pad Thai");
        assert_eq!(item.restaurant_id, r.id);

        // Update
        let updated = repo
            .update_menu_item(
                item.id,
                openwok_core::types::UpdateMenuItemRequest {
                    name: Some("Pad See Ew".into()),
                    price: Some(Money::from("13.99")),
                },
            )
            .await
            .unwrap();
        assert_eq!(updated.name, "Pad See Ew");
        assert_eq!(updated.price, Money::from("13.99"));

        // Delete
        repo.delete_menu_item(item.id).await.unwrap();

        // Verify deleted
        let result = repo.delete_menu_item(item.id).await;
        assert!(matches!(result, Err(RepoError::NotFound)));
    }

    #[tokio::test]
    async fn update_user_role() {
        let repo = test_repo();
        let user = repo
            .create_user(CreateUserRequest {
                supabase_user_id: "role_test".into(),
                email: "role@test.com".into(),
                name: None,
                role: None,
            })
            .await
            .unwrap();
        assert_eq!(user.role, UserRole::Customer);

        let updated = repo
            .update_user_role(user.id, UserRole::RestaurantOwner)
            .await
            .unwrap();
        assert_eq!(updated.role, UserRole::RestaurantOwner);
    }

    #[tokio::test]
    async fn add_menu_item_nonexistent_restaurant() {
        let repo = test_repo();
        let result = repo
            .add_menu_item(
                RestaurantId::new(),
                CreateMenuItemRequest {
                    name: "Ghost Item".into(),
                    price: Money::from("10.00"),
                },
            )
            .await;
        assert!(matches!(result, Err(RepoError::NotFound)));
    }
}
