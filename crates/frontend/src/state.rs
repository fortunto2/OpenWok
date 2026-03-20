#![allow(non_snake_case)]

use openwok_core::money::Money;

// --- Auth state ---

#[derive(Clone, Default, PartialEq)]
pub struct UserState {
    pub jwt: Option<String>,
    pub email: Option<String>,
}

pub fn get_jwt_from_storage() -> Option<String> {
    get_local_storage()?.get_item("openwok_jwt").ok()?
}

pub fn get_local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

pub fn save_jwt_to_storage(jwt: &str) {
    if let Some(storage) = get_local_storage() {
        let _ = storage.set_item("openwok_jwt", jwt);
    }
}

pub fn clear_jwt_from_storage() {
    if let Some(storage) = get_local_storage() {
        let _ = storage.remove_item("openwok_jwt");
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
