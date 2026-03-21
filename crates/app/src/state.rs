#![allow(non_snake_case)]

use openwok_core::money::Money;

// --- Auth state (simplified for fullstack — server handles JWT) ---

#[derive(Clone, Default, PartialEq)]
pub struct UserState {
    pub jwt: Option<String>,
    pub email: Option<String>,
}

#[cfg(target_arch = "wasm32")]
pub fn get_jwt_from_storage() -> Option<String> {
    let storage = web_sys::window()?.local_storage().ok()??;
    storage.get_item("openwok_jwt").ok()?
}

#[cfg(target_arch = "wasm32")]
pub fn save_jwt_to_storage(jwt: &str) {
    if let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item("openwok_jwt", jwt);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn clear_jwt_from_storage() {
    if let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
    {
        let _ = storage.remove_item("openwok_jwt");
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_jwt_from_storage() -> Option<String> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_jwt_to_storage(_jwt: &str) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn clear_jwt_from_storage() {}

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
