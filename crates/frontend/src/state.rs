#![allow(non_snake_case)]

use openwok_core::money::Money;

// --- Auth state ---

#[derive(Clone, Default, PartialEq)]
pub struct UserState {
    pub jwt: Option<String>,
    pub email: Option<String>,
}

// --- JWT storage: web_sys on WASM, file-based on native ---

#[cfg(target_arch = "wasm32")]
pub fn get_jwt_from_storage() -> Option<String> {
    let storage = web_sys::window()?.local_storage().ok()??;
    storage.get_item("openwok_jwt").ok()?
}

#[cfg(target_arch = "wasm32")]
pub fn save_jwt_to_storage(jwt: &str) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item("openwok_jwt", jwt);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn clear_jwt_from_storage() {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.remove_item("openwok_jwt");
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_jwt_from_storage() -> Option<String> {
    let path = dirs::data_dir()?
        .join("co.superduperai.openwok")
        .join("jwt.txt");
    std::fs::read_to_string(&path)
        .ok()
        .filter(|s| !s.is_empty())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_jwt_to_storage(jwt: &str) {
    if let Some(dir) = dirs::data_dir() {
        let dir = dir.join("co.superduperai.openwok");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("jwt.txt"), jwt);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn clear_jwt_from_storage() {
    if let Some(dir) = dirs::data_dir() {
        let _ = std::fs::remove_file(dir.join("co.superduperai.openwok").join("jwt.txt"));
    }
}

// --- App mode (Customer / Courier) ---

#[derive(Clone, Debug, Default, PartialEq)]
pub enum AppMode {
    #[default]
    Customer,
    Courier,
}

// --- Platform config (fetched from API) ---

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
