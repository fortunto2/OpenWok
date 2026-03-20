use crate::money::Money;
use crate::pricing::{PricingBreakdown, calculate_pricing};
use crate::types::{CourierId, MenuItemId, OrderId, RestaurantId, ZoneId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Created,
    Confirmed,
    Preparing,
    ReadyForPickup,
    InDelivery,
    Delivered,
    Cancelled,
}

impl OrderStatus {
    pub fn valid_transitions(self) -> &'static [OrderStatus] {
        use OrderStatus::*;
        match self {
            Created => &[Confirmed, Cancelled],
            Confirmed => &[Preparing, Cancelled],
            Preparing => &[ReadyForPickup, Cancelled],
            ReadyForPickup => &[InDelivery],
            InDelivery => &[Delivered],
            Delivered => &[],
            Cancelled => &[],
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OrderError {
    #[error("invalid transition from {from:?} to {to:?}")]
    InvalidTransition { from: OrderStatus, to: OrderStatus },
    #[error("order has no items")]
    EmptyOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub menu_item_id: MenuItemId,
    pub name: String,
    pub quantity: u32,
    pub unit_price: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub items: Vec<OrderItem>,
    pub restaurant_id: RestaurantId,
    pub courier_id: Option<CourierId>,
    pub customer_address: String,
    pub zone_id: ZoneId,
    pub status: OrderStatus,
    pub pricing: PricingBreakdown,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_eta: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_delivery_at: Option<DateTime<Utc>>,
}

impl Order {
    pub fn new(
        items: Vec<OrderItem>,
        restaurant_id: RestaurantId,
        customer_address: String,
        zone_id: ZoneId,
        delivery_fee: Money,
        tip: Money,
        local_ops_fee: Money,
    ) -> Result<Self, OrderError> {
        if items.is_empty() {
            return Err(OrderError::EmptyOrder);
        }

        let food_total = items.iter().fold(Money::zero(), |acc, item| {
            acc + item.unit_price * rust_decimal::Decimal::from(item.quantity)
        });

        let pricing = calculate_pricing(food_total, delivery_fee, tip, local_ops_fee);
        let now = Utc::now();

        Ok(Self {
            id: OrderId::new(),
            items,
            restaurant_id,
            courier_id: None,
            customer_address,
            zone_id,
            status: OrderStatus::Created,
            pricing,
            created_at: now,
            updated_at: now,
            estimated_eta: None,
            actual_delivery_at: None,
        })
    }

    pub fn transition(&mut self, new_status: OrderStatus) -> Result<(), OrderError> {
        if self.status.valid_transitions().contains(&new_status) {
            self.status = new_status;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(OrderError::InvalidTransition {
                from: self.status,
                to: new_status,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_items() -> Vec<OrderItem> {
        vec![OrderItem {
            menu_item_id: MenuItemId::new(),
            name: "Pad Thai".into(),
            quantity: 2,
            unit_price: Money::from("12.50"),
        }]
    }

    fn sample_order() -> Order {
        Order::new(
            sample_items(),
            RestaurantId::new(),
            "123 Main St".into(),
            ZoneId::new(),
            Money::from("5.00"),
            Money::from("3.00"),
            Money::from("2.50"),
        )
        .unwrap()
    }

    // --- State machine tests ---

    #[test]
    fn valid_full_flow() {
        let mut o = sample_order();
        assert_eq!(o.status, OrderStatus::Created);
        o.transition(OrderStatus::Confirmed).unwrap();
        o.transition(OrderStatus::Preparing).unwrap();
        o.transition(OrderStatus::ReadyForPickup).unwrap();
        o.transition(OrderStatus::InDelivery).unwrap();
        o.transition(OrderStatus::Delivered).unwrap();
        assert_eq!(o.status, OrderStatus::Delivered);
    }

    #[test]
    fn cancel_from_created() {
        let mut o = sample_order();
        o.transition(OrderStatus::Cancelled).unwrap();
        assert_eq!(o.status, OrderStatus::Cancelled);
    }

    #[test]
    fn cancel_from_confirmed() {
        let mut o = sample_order();
        o.transition(OrderStatus::Confirmed).unwrap();
        o.transition(OrderStatus::Cancelled).unwrap();
        assert_eq!(o.status, OrderStatus::Cancelled);
    }

    #[test]
    fn cancel_from_preparing() {
        let mut o = sample_order();
        o.transition(OrderStatus::Confirmed).unwrap();
        o.transition(OrderStatus::Preparing).unwrap();
        o.transition(OrderStatus::Cancelled).unwrap();
        assert_eq!(o.status, OrderStatus::Cancelled);
    }

    #[test]
    fn cannot_cancel_from_ready_for_pickup() {
        let mut o = sample_order();
        o.transition(OrderStatus::Confirmed).unwrap();
        o.transition(OrderStatus::Preparing).unwrap();
        o.transition(OrderStatus::ReadyForPickup).unwrap();
        assert!(o.transition(OrderStatus::Cancelled).is_err());
    }

    #[test]
    fn cannot_cancel_from_in_delivery() {
        let mut o = sample_order();
        o.transition(OrderStatus::Confirmed).unwrap();
        o.transition(OrderStatus::Preparing).unwrap();
        o.transition(OrderStatus::ReadyForPickup).unwrap();
        o.transition(OrderStatus::InDelivery).unwrap();
        assert!(o.transition(OrderStatus::Cancelled).is_err());
    }

    #[test]
    fn cannot_skip_states() {
        let mut o = sample_order();
        assert!(o.transition(OrderStatus::Preparing).is_err());
    }

    #[test]
    fn cannot_transition_from_delivered() {
        let mut o = sample_order();
        o.transition(OrderStatus::Confirmed).unwrap();
        o.transition(OrderStatus::Preparing).unwrap();
        o.transition(OrderStatus::ReadyForPickup).unwrap();
        o.transition(OrderStatus::InDelivery).unwrap();
        o.transition(OrderStatus::Delivered).unwrap();
        assert!(o.transition(OrderStatus::Created).is_err());
    }

    #[test]
    fn cannot_transition_from_cancelled() {
        let mut o = sample_order();
        o.transition(OrderStatus::Cancelled).unwrap();
        assert!(o.transition(OrderStatus::Confirmed).is_err());
    }

    #[test]
    fn transition_updates_timestamp() {
        let mut o = sample_order();
        let before = o.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        o.transition(OrderStatus::Confirmed).unwrap();
        assert!(o.updated_at >= before);
    }

    // --- Order::new tests ---

    #[test]
    fn new_order_starts_as_created() {
        let o = sample_order();
        assert_eq!(o.status, OrderStatus::Created);
    }

    #[test]
    fn new_order_calculates_pricing() {
        let o = sample_order();
        // 2 * $12.50 = $25.00 food
        assert_eq!(o.pricing.food_total, Money::from("25.00"));
        assert_eq!(o.pricing.federal_fee, Money::from("1.00"));
        assert_eq!(o.pricing.delivery_fee, Money::from("5.00"));
        assert_eq!(o.pricing.tip, Money::from("3.00"));
        assert_eq!(o.pricing.local_ops_fee, Money::from("2.50"));
    }

    #[test]
    fn new_order_no_courier() {
        let o = sample_order();
        assert!(o.courier_id.is_none());
    }

    #[test]
    fn empty_order_rejected() {
        let result = Order::new(
            vec![],
            RestaurantId::new(),
            "123 Main St".into(),
            ZoneId::new(),
            Money::from("5.00"),
            Money::from("0.00"),
            Money::from("2.50"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn order_serde_roundtrip() {
        let o = sample_order();
        let json = serde_json::to_string(&o).unwrap();
        let back: Order = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, OrderStatus::Created);
        assert_eq!(back.items.len(), 1);
    }
}
