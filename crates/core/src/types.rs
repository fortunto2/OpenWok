use crate::money::Money;
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
                Self(Uuid::new_v4())
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
}
