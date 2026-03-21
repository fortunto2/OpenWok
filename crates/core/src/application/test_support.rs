use std::sync::Mutex;

use async_trait::async_trait;
use chrono::Utc;

use crate::money::Money;
use crate::order::{Order, OrderItem, OrderStatus};
use crate::repo::{
    AdminMetrics, AssignCourierResult, CourierUtilization, CreateCourierRequest,
    CreateMenuItemRequest, CreateOrderRequest, CreateRestaurantRequest, PublicEconomics, RepoError,
    Repository, RevenueBreakdown,
};
use crate::types::{
    Courier, CourierId, CourierKind, CreatePaymentRequest, CreateUserRequest, Dispute, DisputeId,
    DisputeStatus, MenuItem, MenuItemId, OrderId, Payment, PaymentId, PaymentStatus, Restaurant,
    RestaurantId, UpdateMenuItemRequest, UpdatePaymentStatusRequest, UpdateRestaurantRequest, User,
    UserId, UserRole, ZoneId,
};

pub(super) struct TestRepo {
    orders: Mutex<Vec<Order>>,
    payments: Mutex<Vec<Payment>>,
    couriers: Mutex<Vec<Courier>>,
}

impl TestRepo {
    pub(super) fn with_preparing_order_and_available_courier() -> (Self, OrderId) {
        let zone_id = ZoneId::new();
        let mut order = sample_order(zone_id);
        order.status = OrderStatus::Preparing;

        let courier = Courier {
            id: CourierId::new(),
            name: "Courier".into(),
            kind: CourierKind::Human,
            zone_id,
            available: true,
            user_id: None,
        };

        let order_id = order.id;
        (
            Self {
                orders: Mutex::new(vec![order]),
                payments: Mutex::new(vec![]),
                couriers: Mutex::new(vec![courier]),
            },
            order_id,
        )
    }

    pub(super) fn with_created_order_and_payment(payment_status: PaymentStatus) -> (Self, OrderId) {
        let zone_id = ZoneId::new();
        let order = sample_order(zone_id);
        let order_id = order.id;
        let payment = sample_payment(order_id, payment_status, None);

        (
            Self {
                orders: Mutex::new(vec![order]),
                payments: Mutex::new(vec![payment]),
                couriers: Mutex::new(vec![]),
            },
            order_id,
        )
    }

    pub(super) fn order(&self, id: OrderId) -> Order {
        self.orders
            .lock()
            .unwrap()
            .iter()
            .find(|order| order.id == id)
            .cloned()
            .unwrap()
    }

    pub(super) fn payment_by_order(&self, order_id: OrderId) -> Payment {
        self.payments
            .lock()
            .unwrap()
            .iter()
            .find(|payment| payment.order_id == order_id)
            .cloned()
            .unwrap()
    }
}

fn sample_order(zone_id: ZoneId) -> Order {
    Order::new(
        vec![OrderItem {
            menu_item_id: MenuItemId::new(),
            name: "Pad Thai".into(),
            quantity: 1,
            unit_price: Money::from("10.00"),
        }],
        RestaurantId::new(),
        "123 Test St".into(),
        zone_id,
        Money::from("5.00"),
        Money::from("2.00"),
        Money::from("2.50"),
    )
    .unwrap()
}

fn sample_payment(
    order_id: OrderId,
    status: PaymentStatus,
    stripe_payment_intent_id: Option<String>,
) -> Payment {
    Payment {
        id: PaymentId::new(),
        order_id,
        stripe_payment_intent_id,
        stripe_checkout_session_id: None,
        status,
        amount_total: Money::from("20.50"),
        restaurant_amount: Money::from("10.00"),
        courier_amount: Money::from("7.00"),
        federal_amount: Money::from("1.00"),
        local_ops_amount: Money::from("2.50"),
        processing_amount: Money::from("0.00"),
        created_at: Utc::now(),
    }
}

#[async_trait]
impl Repository for TestRepo {
    async fn list_restaurants(&self) -> Result<Vec<Restaurant>, RepoError> {
        Ok(vec![])
    }

    async fn get_restaurant(&self, _id: RestaurantId) -> Result<Restaurant, RepoError> {
        Err(RepoError::NotFound)
    }

    async fn create_restaurant(
        &self,
        _req: CreateRestaurantRequest,
    ) -> Result<Restaurant, RepoError> {
        Err(RepoError::NotFound)
    }

    async fn list_orders(&self) -> Result<Vec<Order>, RepoError> {
        Ok(self.orders.lock().unwrap().clone())
    }

    async fn get_order(&self, id: OrderId) -> Result<Order, RepoError> {
        self.orders
            .lock()
            .unwrap()
            .iter()
            .find(|order| order.id == id)
            .cloned()
            .ok_or(RepoError::NotFound)
    }

    async fn create_order(&self, req: CreateOrderRequest) -> Result<Order, RepoError> {
        let order = Order::new(
            req.items
                .into_iter()
                .map(|item| OrderItem {
                    menu_item_id: item.menu_item_id,
                    name: item.name,
                    quantity: item.quantity,
                    unit_price: item.unit_price,
                })
                .collect(),
            req.restaurant_id,
            req.customer_address,
            req.zone_id,
            req.delivery_fee,
            req.tip,
            req.local_ops_fee,
        )
        .map_err(|error| RepoError::Internal(error.to_string()))?;

        self.orders.lock().unwrap().push(order.clone());
        Ok(order)
    }

    async fn update_order_status(
        &self,
        id: OrderId,
        status: OrderStatus,
    ) -> Result<Order, RepoError> {
        let mut orders = self.orders.lock().unwrap();
        let order = orders
            .iter_mut()
            .find(|order| order.id == id)
            .ok_or(RepoError::NotFound)?;

        order
            .transition(status)
            .map_err(|error| RepoError::InvalidTransition(error.to_string()))?;

        Ok(order.clone())
    }

    async fn assign_courier(&self, order_id: OrderId) -> Result<AssignCourierResult, RepoError> {
        let mut orders = self.orders.lock().unwrap();
        let order = orders
            .iter_mut()
            .find(|order| order.id == order_id)
            .ok_or(RepoError::NotFound)?;

        let mut couriers = self.couriers.lock().unwrap();
        let courier = couriers
            .iter_mut()
            .find(|courier| courier.zone_id == order.zone_id && courier.available)
            .ok_or_else(|| RepoError::Conflict("no available courier in zone".into()))?;

        courier.available = false;
        order.courier_id = Some(courier.id);
        order.updated_at = Utc::now();

        Ok(AssignCourierResult {
            order_id: order_id.to_string(),
            courier_id: courier.id.to_string(),
        })
    }

    async fn list_couriers(&self) -> Result<Vec<Courier>, RepoError> {
        Ok(self.couriers.lock().unwrap().clone())
    }

    async fn create_courier(&self, _req: CreateCourierRequest) -> Result<Courier, RepoError> {
        Err(RepoError::NotFound)
    }

    async fn toggle_courier_available(
        &self,
        _id: CourierId,
        _available: bool,
    ) -> Result<Courier, RepoError> {
        Err(RepoError::NotFound)
    }

    async fn create_user(&self, _req: CreateUserRequest) -> Result<User, RepoError> {
        Err(RepoError::NotFound)
    }

    async fn get_user(&self, _id: UserId) -> Result<User, RepoError> {
        Err(RepoError::NotFound)
    }

    async fn get_user_by_supabase_id(&self, _supabase_user_id: &str) -> Result<User, RepoError> {
        Err(RepoError::NotFound)
    }

    async fn create_payment(&self, req: CreatePaymentRequest) -> Result<Payment, RepoError> {
        let payment = Payment {
            id: PaymentId::new(),
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
            created_at: Utc::now(),
        };

        self.payments.lock().unwrap().push(payment.clone());
        Ok(payment)
    }

    async fn get_payment_by_order(&self, order_id: OrderId) -> Result<Payment, RepoError> {
        self.payments
            .lock()
            .unwrap()
            .iter()
            .find(|payment| payment.order_id == order_id)
            .cloned()
            .ok_or(RepoError::NotFound)
    }

    async fn update_payment_status(
        &self,
        id: PaymentId,
        req: UpdatePaymentStatusRequest,
    ) -> Result<Payment, RepoError> {
        let mut payments = self.payments.lock().unwrap();
        let payment = payments
            .iter_mut()
            .find(|payment| payment.id == id)
            .ok_or(RepoError::NotFound)?;

        payment.status = req.status;
        if req.stripe_payment_intent_id.is_some() {
            payment.stripe_payment_intent_id = req.stripe_payment_intent_id;
        }

        Ok(payment.clone())
    }

    async fn update_restaurant(
        &self,
        _id: RestaurantId,
        _req: UpdateRestaurantRequest,
    ) -> Result<Restaurant, RepoError> {
        Err(RepoError::NotFound)
    }

    async fn toggle_restaurant_active(
        &self,
        _id: RestaurantId,
        _active: bool,
    ) -> Result<Restaurant, RepoError> {
        Err(RepoError::NotFound)
    }

    async fn list_restaurants_by_owner(
        &self,
        _user_id: UserId,
    ) -> Result<Vec<Restaurant>, RepoError> {
        Ok(vec![])
    }

    async fn add_menu_item(
        &self,
        _restaurant_id: RestaurantId,
        _req: CreateMenuItemRequest,
    ) -> Result<MenuItem, RepoError> {
        Err(RepoError::NotFound)
    }

    async fn get_menu_item(&self, _id: MenuItemId) -> Result<MenuItem, RepoError> {
        Err(RepoError::NotFound)
    }

    async fn update_menu_item(
        &self,
        _id: MenuItemId,
        _req: UpdateMenuItemRequest,
    ) -> Result<MenuItem, RepoError> {
        Err(RepoError::NotFound)
    }

    async fn delete_menu_item(&self, _id: MenuItemId) -> Result<(), RepoError> {
        Err(RepoError::NotFound)
    }

    async fn update_user_role(&self, _user_id: UserId, _role: UserRole) -> Result<User, RepoError> {
        Err(RepoError::NotFound)
    }

    async fn get_courier_by_user_id(&self, _user_id: &str) -> Result<Courier, RepoError> {
        Err(RepoError::NotFound)
    }

    async fn list_courier_orders(&self, _courier_id: CourierId) -> Result<Vec<Order>, RepoError> {
        Ok(vec![])
    }

    async fn list_restaurant_orders(
        &self,
        _restaurant_id: RestaurantId,
    ) -> Result<Vec<Order>, RepoError> {
        Ok(vec![])
    }

    async fn get_economics(&self) -> Result<PublicEconomics, RepoError> {
        Ok(PublicEconomics {
            total_orders: 0,
            total_food_revenue: "0.00".into(),
            total_delivery_fees: "0.00".into(),
            total_federal_fees: "0.00".into(),
            total_local_ops_fees: "0.00".into(),
            total_processing_fees: "0.00".into(),
            avg_order_value: "0.00".into(),
        })
    }

    async fn get_metrics(&self) -> Result<AdminMetrics, RepoError> {
        Ok(AdminMetrics {
            order_count: 0,
            orders_by_status: std::collections::HashMap::new(),
            on_time_delivery_rate: 0.0,
            avg_eta_error_minutes: 0.0,
            revenue_breakdown: RevenueBreakdown {
                total_food_revenue: "0.00".into(),
                total_delivery_fees: "0.00".into(),
                total_federal_fees: "0.00".into(),
                total_local_ops_fees: "0.00".into(),
                total_processing_fees: "0.00".into(),
            },
            courier_utilization: CourierUtilization {
                available: 0,
                total: 0,
            },
            orders_by_zone: std::collections::HashMap::new(),
        })
    }

    async fn list_users(&self) -> Result<Vec<User>, RepoError> {
        Ok(vec![])
    }

    async fn set_user_blocked(&self, _user_id: UserId, _blocked: bool) -> Result<User, RepoError> {
        Err(RepoError::NotFound)
    }

    async fn create_dispute(
        &self,
        _order_id: OrderId,
        _user_id: UserId,
        _reason: String,
    ) -> Result<Dispute, RepoError> {
        Err(RepoError::NotFound)
    }

    async fn list_disputes(&self) -> Result<Vec<Dispute>, RepoError> {
        Ok(vec![])
    }

    async fn resolve_dispute(
        &self,
        _id: DisputeId,
        _status: DisputeStatus,
        _resolution: Option<String>,
    ) -> Result<Dispute, RepoError> {
        Err(RepoError::NotFound)
    }
}
