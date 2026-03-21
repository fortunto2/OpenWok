#![allow(non_snake_case)]

use openwok_core::money::Money;

// --- Auth state (simplified for fullstack — server handles JWT) ---

#[derive(Clone, Default, PartialEq)]
pub struct UserState {
    pub jwt: Option<String>,
    pub email: Option<String>,
}

// --- App mode (Customer / Courier) ---

#[derive(Clone, Debug, Default, PartialEq)]
pub enum AppMode {
    #[default]
    Customer,
    Courier,
}

// --- Platform config (loaded via server fn) ---

#[derive(Clone, PartialEq)]
pub struct PlatformConfig {
    pub delivery_fee: String,
    pub local_ops_fee: String,
    pub federal_fee: String,
    pub default_tip: String,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            delivery_fee: "5.00".into(),
            local_ops_fee: "2.50".into(),
            federal_fee: "1.00".into(),
            default_tip: "3.00".into(),
        }
    }
}

// --- Cart state ---

#[derive(Clone, PartialEq)]
pub struct CartItem {
    pub menu_item_id: String,
    pub name: String,
    pub price: Money,
    pub quantity: u32,
}

#[derive(Clone, Default, PartialEq)]
pub struct CartState {
    pub items: Vec<CartItem>,
    pub restaurant_id: String,
    pub restaurant_name: String,
    pub zone_id: String,
}
