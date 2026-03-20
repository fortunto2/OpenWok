#![allow(non_snake_case)]

use openwok_core::money::Money;

// --- App mode ---

#[derive(Clone, Debug, Default, PartialEq)]
pub enum AppMode {
    #[default]
    Customer,
    Courier,
}

// --- Auth state ---

#[derive(Clone, Default, PartialEq)]
pub struct UserState {
    pub jwt: Option<String>,
    pub email: Option<String>,
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
