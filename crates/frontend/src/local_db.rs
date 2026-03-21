#![allow(non_snake_case)]

//! Platform-abstracted local key-value cache.
//! WASM: localStorage (web_sys::Storage). Native: JSON files (dirs crate).
//! Simple key-value: set/get/delete with serde_json::Value.

use serde_json::Value;

// ====== WASM: localStorage ======

#[cfg(target_arch = "wasm32")]
const KEY_PREFIX: &str = "openwok_cache_";

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

#[cfg(target_arch = "wasm32")]
pub fn get(key: &str) -> Option<Value> {
    let s = storage()?;
    let raw = s.get_item(&format!("{KEY_PREFIX}{key}")).ok()??;
    serde_json::from_str(&raw).ok()
}

#[cfg(target_arch = "wasm32")]
pub fn set(key: &str, value: &Value) {
    if let Some(s) = storage() {
        let json = serde_json::to_string(value).unwrap_or_default();
        let _ = s.set_item(&format!("{KEY_PREFIX}{key}"), &json);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn delete(key: &str) {
    if let Some(s) = storage() {
        let _ = s.remove_item(&format!("{KEY_PREFIX}{key}"));
    }
}

// ====== Native: JSON files ======

#[cfg(not(target_arch = "wasm32"))]
fn cache_dir() -> Option<std::path::PathBuf> {
    dirs::data_dir().map(|d| d.join("co.superduperai.openwok").join("cache"))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get(key: &str) -> Option<Value> {
    let path = cache_dir()?.join(format!("{key}.json"));
    let data = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set(key: &str, value: &Value) {
    if let Some(dir) = cache_dir() {
        let _ = std::fs::create_dir_all(&dir);
        let json = serde_json::to_string(value).unwrap_or_default();
        let _ = std::fs::write(dir.join(format!("{key}.json")), json);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn delete(key: &str) {
    if let Some(dir) = cache_dir() {
        let _ = std::fs::remove_file(dir.join(format!("{key}.json")));
    }
}
