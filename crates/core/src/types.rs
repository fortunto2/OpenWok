use crate::money::Money;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! id_newtype {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, utoipa::ToSchema,
        )]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

id_newtype!(RestaurantId);
id_newtype!(CourierId);
id_newtype!(OrderId);
id_newtype!(NodeId);
id_newtype!(ZoneId);
id_newtype!(MenuItemId);
id_newtype!(UserId);
id_newtype!(PaymentId);
id_newtype!(DisputeId);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MenuItem {
    pub id: MenuItemId,
    pub name: String,
    pub price: Money,
    pub restaurant_id: RestaurantId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Restaurant {
    pub id: RestaurantId,
    pub name: String,
    pub zone_id: ZoneId,
    pub menu: Vec<MenuItem>,
    pub active: bool,
    pub owner_id: Option<UserId>,
    pub description: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Zone {
    pub id: ZoneId,
    pub name: String,
    pub node_id: NodeId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Node {
    pub id: NodeId,
    pub name: String,
    pub local_ops_fee: Money,
    pub zones: Vec<ZoneId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub enum CourierKind {
    Human,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Courier {
    pub id: CourierId,
    pub name: String,
    pub kind: CourierKind,
    pub zone_id: ZoneId,
    pub available: bool,
    pub user_id: Option<String>,
}

// --- Auth & Payments ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub enum UserRole {
    Customer,
    RestaurantOwner,
    Courier,
    NodeOperator,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Customer => write!(f, "Customer"),
            UserRole::RestaurantOwner => write!(f, "RestaurantOwner"),
            UserRole::Courier => write!(f, "Courier"),
            UserRole::NodeOperator => write!(f, "NodeOperator"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Customer" => Ok(UserRole::Customer),
            "RestaurantOwner" => Ok(UserRole::RestaurantOwner),
            "Courier" => Ok(UserRole::Courier),
            "NodeOperator" => Ok(UserRole::NodeOperator),
            _ => Err(format!("unknown role: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct User {
    pub id: UserId,
    pub supabase_user_id: String,
    pub email: String,
    pub name: Option<String>,
    pub role: UserRole,
    pub blocked: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub enum PaymentStatus {
    Pending,
    Succeeded,
    Failed,
    Refunded,
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentStatus::Pending => write!(f, "Pending"),
            PaymentStatus::Succeeded => write!(f, "Succeeded"),
            PaymentStatus::Failed => write!(f, "Failed"),
            PaymentStatus::Refunded => write!(f, "Refunded"),
        }
    }
}

impl std::str::FromStr for PaymentStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(PaymentStatus::Pending),
            "Succeeded" => Ok(PaymentStatus::Succeeded),
            "Failed" => Ok(PaymentStatus::Failed),
            "Refunded" => Ok(PaymentStatus::Refunded),
            _ => Err(format!("unknown payment status: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Payment {
    pub id: PaymentId,
    pub order_id: OrderId,
    pub stripe_payment_intent_id: Option<String>,
    pub stripe_checkout_session_id: Option<String>,
    pub status: PaymentStatus,
    pub amount_total: Money,
    pub restaurant_amount: Money,
    pub courier_amount: Money,
    pub federal_amount: Money,
    pub local_ops_amount: Money,
    pub processing_amount: Money,
    pub created_at: DateTime<Utc>,
}

// --- Disputes ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub enum DisputeStatus {
    Open,
    Resolved,
    Dismissed,
}

impl std::fmt::Display for DisputeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisputeStatus::Open => write!(f, "Open"),
            DisputeStatus::Resolved => write!(f, "Resolved"),
            DisputeStatus::Dismissed => write!(f, "Dismissed"),
        }
    }
}

impl std::str::FromStr for DisputeStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Open" => Ok(DisputeStatus::Open),
            "Resolved" => Ok(DisputeStatus::Resolved),
            "Dismissed" => Ok(DisputeStatus::Dismissed),
            _ => Err(format!("unknown dispute status: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Dispute {
    pub id: DisputeId,
    pub order_id: OrderId,
    pub user_id: UserId,
    pub reason: String,
    pub status: DisputeStatus,
    pub resolution: Option<String>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

// --- Request types ---

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateUserRequest {
    pub supabase_user_id: String,
    pub email: String,
    pub name: Option<String>,
    pub role: Option<UserRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreatePaymentRequest {
    pub order_id: OrderId,
    pub stripe_checkout_session_id: Option<String>,
    pub amount_total: Money,
    pub restaurant_amount: Money,
    pub courier_amount: Money,
    pub federal_amount: Money,
    pub local_ops_amount: Money,
    pub processing_amount: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdatePaymentStatusRequest {
    pub status: PaymentStatus,
    pub stripe_payment_intent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateRestaurantRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateMenuItemRequest {
    pub name: Option<String>,
    pub price: Option<Money>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateDisputeRequest {
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ResolveDisputeRequest {
    pub status: DisputeStatus,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SetBlockedRequest {
    pub blocked: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_newtypes_are_unique() {
        let a = RestaurantId::new();
        let b = RestaurantId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn restaurant_serde_roundtrip() {
        let r = Restaurant {
            id: RestaurantId::new(),
            name: "Test Wok".into(),
            zone_id: ZoneId::new(),
            menu: vec![MenuItem {
                id: MenuItemId::new(),
                name: "Pad Thai".into(),
                price: Money::from("12.99"),
                restaurant_id: RestaurantId::new(),
            }],
            active: true,
            owner_id: None,
            description: None,
            address: None,
            phone: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: Restaurant = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Test Wok");
        assert_eq!(back.menu.len(), 1);
    }

    #[test]
    fn courier_serde_roundtrip() {
        let c = Courier {
            id: CourierId::new(),
            name: "Alex".into(),
            kind: CourierKind::Human,
            zone_id: ZoneId::new(),
            available: true,
            user_id: None,
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: Courier = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Alex");
        assert!(back.available);
    }

    #[test]
    fn node_serde_roundtrip() {
        let n = Node {
            id: NodeId::new(),
            name: "LA Node".into(),
            local_ops_fee: Money::from("2.50"),
            zones: vec![ZoneId::new()],
        };
        let json = serde_json::to_string(&n).unwrap();
        let back: Node = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "LA Node");
    }

    #[test]
    fn user_serde_roundtrip() {
        let u = User {
            id: UserId::new(),
            supabase_user_id: "sub_123".into(),
            email: "test@example.com".into(),
            name: Some("Test User".into()),
            role: UserRole::Customer,
            blocked: false,
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&u).unwrap();
        let back: User = serde_json::from_str(&json).unwrap();
        assert_eq!(back.email, "test@example.com");
        assert_eq!(back.role, UserRole::Customer);
    }

    #[test]
    fn user_role_display_and_parse() {
        assert_eq!(UserRole::Customer.to_string(), "Customer");
        assert_eq!(UserRole::RestaurantOwner.to_string(), "RestaurantOwner");
        assert_eq!("Customer".parse::<UserRole>().unwrap(), UserRole::Customer);
        assert_eq!(
            "RestaurantOwner".parse::<UserRole>().unwrap(),
            UserRole::RestaurantOwner
        );
        assert!("Unknown".parse::<UserRole>().is_err());
    }

    #[test]
    fn payment_serde_roundtrip() {
        let p = Payment {
            id: PaymentId::new(),
            order_id: OrderId::new(),
            stripe_payment_intent_id: None,
            stripe_checkout_session_id: Some("cs_test_123".into()),
            status: PaymentStatus::Pending,
            amount_total: Money::from("37.86"),
            restaurant_amount: Money::from("25.00"),
            courier_amount: Money::from("8.00"),
            federal_amount: Money::from("1.00"),
            local_ops_amount: Money::from("2.50"),
            processing_amount: Money::from("1.36"),
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: Payment = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, PaymentStatus::Pending);
        assert_eq!(back.amount_total, Money::from("37.86"));
    }

    #[test]
    fn payment_status_display_and_parse() {
        assert_eq!(PaymentStatus::Pending.to_string(), "Pending");
        assert_eq!(PaymentStatus::Succeeded.to_string(), "Succeeded");
        assert_eq!(
            "Failed".parse::<PaymentStatus>().unwrap(),
            PaymentStatus::Failed
        );
        assert!("Invalid".parse::<PaymentStatus>().is_err());
    }

    #[test]
    fn user_id_and_payment_id_unique() {
        let a = UserId::new();
        let b = UserId::new();
        assert_ne!(a, b);
        let c = PaymentId::new();
        let d = PaymentId::new();
        assert_ne!(c, d);
    }
}
