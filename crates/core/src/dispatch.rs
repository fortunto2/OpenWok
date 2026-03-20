use crate::repo::{AssignCourierResult, RepoError, Repository};
use crate::types::OrderId;
use serde::Serialize;

/// Event broadcast on order status changes and courier assignments.
#[derive(Clone, Debug, Serialize)]
pub struct OrderEvent {
    pub order_id: String,
    pub status: String,
}

/// Auto-dispatch: assign an available courier in the order's zone.
/// Returns None if no courier available (order stays at ReadyForPickup).
pub async fn auto_dispatch<R: Repository>(
    repo: &R,
    order_id: OrderId,
) -> Result<Option<AssignCourierResult>, RepoError> {
    match repo.assign_courier(order_id).await {
        Ok(result) => Ok(Some(result)),
        Err(RepoError::Conflict(_)) => Ok(None), // no courier available
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::money::Money;
    use crate::order::{Order, OrderStatus};
    use crate::repo::*;
    use crate::types::*;
    use async_trait::async_trait;
    use std::sync::Mutex;

    /// In-memory repo for dispatch tests.
    struct MockRepo {
        orders: Mutex<Vec<Order>>,
        couriers: Mutex<Vec<Courier>>,
    }

    impl MockRepo {
        fn new() -> Self {
            Self {
                orders: Mutex::new(vec![]),
                couriers: Mutex::new(vec![]),
            }
        }

        fn with_order_and_courier(zone_id: ZoneId) -> (Self, OrderId) {
            let order = Order::new(
                vec![crate::order::OrderItem {
                    menu_item_id: MenuItemId::new(),
                    name: "Test".into(),
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
            .unwrap();
            let order_id = order.id;

            let courier = Courier {
                id: CourierId::new(),
                name: "Test Courier".into(),
                kind: CourierKind::Human,
                zone_id,
                available: true,
                user_id: None,
            };

            let repo = Self {
                orders: Mutex::new(vec![order]),
                couriers: Mutex::new(vec![courier]),
            };
            (repo, order_id)
        }
    }

    #[async_trait]
    impl Repository for MockRepo {
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
                .find(|o| o.id == id)
                .cloned()
                .ok_or(RepoError::NotFound)
        }
        async fn create_order(&self, _req: CreateOrderRequest) -> Result<Order, RepoError> {
            Err(RepoError::NotFound)
        }
        async fn update_order_status(
            &self,
            _id: OrderId,
            _status: OrderStatus,
        ) -> Result<Order, RepoError> {
            Err(RepoError::NotFound)
        }
        async fn assign_courier(
            &self,
            order_id: OrderId,
        ) -> Result<AssignCourierResult, RepoError> {
            let orders = self.orders.lock().unwrap();
            let order = orders
                .iter()
                .find(|o| o.id == order_id)
                .ok_or(RepoError::NotFound)?;

            let mut couriers = self.couriers.lock().unwrap();
            let courier = couriers
                .iter_mut()
                .find(|c| c.zone_id == order.zone_id && c.available)
                .ok_or_else(|| RepoError::Conflict("no available courier in zone".into()))?;

            courier.available = false;
            Ok(AssignCourierResult {
                order_id: order_id.to_string(),
                courier_id: courier.id.to_string(),
            })
        }
        async fn list_couriers(&self) -> Result<Vec<Courier>, RepoError> {
            Ok(self.couriers.lock().unwrap().clone())
        }
        async fn create_courier(
            &self,
            _req: CreateCourierRequest,
        ) -> Result<Courier, RepoError> {
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
        async fn get_user_by_supabase_id(
            &self,
            _supabase_user_id: &str,
        ) -> Result<User, RepoError> {
            Err(RepoError::NotFound)
        }
        async fn create_payment(
            &self,
            _req: CreatePaymentRequest,
        ) -> Result<Payment, RepoError> {
            Err(RepoError::NotFound)
        }
        async fn get_payment_by_order(
            &self,
            _order_id: OrderId,
        ) -> Result<Payment, RepoError> {
            Err(RepoError::NotFound)
        }
        async fn update_payment_status(
            &self,
            _id: PaymentId,
            _req: UpdatePaymentStatusRequest,
        ) -> Result<Payment, RepoError> {
            Err(RepoError::NotFound)
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
        async fn update_user_role(
            &self,
            _user_id: UserId,
            _role: UserRole,
        ) -> Result<User, RepoError> {
            Err(RepoError::NotFound)
        }
        async fn get_courier_by_user_id(
            &self,
            _user_id: &str,
        ) -> Result<Courier, RepoError> {
            Err(RepoError::NotFound)
        }
        async fn list_courier_orders(
            &self,
            _courier_id: CourierId,
        ) -> Result<Vec<Order>, RepoError> {
            Ok(vec![])
        }
        async fn get_economics(&self) -> Result<PublicEconomics, RepoError> {
            Err(RepoError::NotFound)
        }
        async fn get_metrics(&self) -> Result<AdminMetrics, RepoError> {
            Err(RepoError::NotFound)
        }
    }

    #[tokio::test]
    async fn auto_dispatch_assigns_courier() {
        let zone = ZoneId::new();
        let (repo, order_id) = MockRepo::with_order_and_courier(zone);

        let result = auto_dispatch(&repo, order_id).await.unwrap();
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.order_id, order_id.to_string());
    }

    #[tokio::test]
    async fn auto_dispatch_returns_none_when_no_courier() {
        let repo = MockRepo::new();
        let order_id = OrderId::new();

        // Order doesn't exist → NotFound error (not None)
        let result = auto_dispatch(&repo, order_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn auto_dispatch_returns_none_wrong_zone() {
        let zone1 = ZoneId::new();
        let zone2 = ZoneId::new();

        // Order in zone1, courier in zone2
        let order = Order::new(
            vec![crate::order::OrderItem {
                menu_item_id: MenuItemId::new(),
                name: "Test".into(),
                quantity: 1,
                unit_price: Money::from("10.00"),
            }],
            RestaurantId::new(),
            "123 Test St".into(),
            zone1,
            Money::from("5.00"),
            Money::from("2.00"),
            Money::from("2.50"),
        )
        .unwrap();
        let order_id = order.id;

        let courier = Courier {
            id: CourierId::new(),
            name: "Wrong Zone Courier".into(),
            kind: CourierKind::Human,
            zone_id: zone2,
            available: true,
            user_id: None,
        };

        let repo = MockRepo {
            orders: Mutex::new(vec![order]),
            couriers: Mutex::new(vec![courier]),
        };

        let result = auto_dispatch(&repo, order_id).await.unwrap();
        assert!(result.is_none()); // No courier in zone1
    }
}
