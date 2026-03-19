use crate::pricing::PricingBreakdown;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub menu_item_id: MenuItemId,
    pub name: String,
    pub quantity: u32,
    pub unit_price: crate::money::Money,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::money::Money;

    #[test]
    fn order_status_serde_roundtrip() {
        let status = OrderStatus::ReadyForPickup;
        let json = serde_json::to_string(&status).unwrap();
        let back: OrderStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, status);
    }

    #[test]
    fn order_serde_roundtrip() {
        let order = Order {
            id: OrderId::new(),
            items: vec![OrderItem {
                menu_item_id: MenuItemId::new(),
                name: "Pad Thai".into(),
                quantity: 2,
                unit_price: Money::from("12.99"),
            }],
            restaurant_id: RestaurantId::new(),
            courier_id: None,
            customer_address: "123 Main St".into(),
            zone_id: ZoneId::new(),
            status: OrderStatus::Created,
            pricing: PricingBreakdown {
                food_total: Money::from("25.98"),
                delivery_fee: Money::from("5.00"),
                tip: Money::from("3.00"),
                federal_fee: Money::from("1.00"),
                local_ops_fee: Money::from("2.50"),
                processing_fee: Money::from("1.39"),
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&order).unwrap();
        let back: Order = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, OrderStatus::Created);
        assert_eq!(back.items.len(), 1);
    }
}
