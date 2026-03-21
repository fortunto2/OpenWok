#![allow(non_snake_case)]

use openwok_core::money::Money;
use reqwest::Client;
use rust_decimal::Decimal;

use crate::state::{CartItem, get_jwt_from_storage};

/// On native: absolute URL to production.
#[cfg(not(target_arch = "wasm32"))]
pub const API_BASE: &str = "https://openwok.superduperai.co/api";

/// On WASM: resolved at runtime from window.location.origin.
#[cfg(target_arch = "wasm32")]
pub fn api_base() -> String {
    let origin = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_else(|| "http://localhost:8080".to_string());
    format!("{origin}/api")
}

#[cfg(not(target_arch = "wasm32"))]
pub fn api_base() -> String {
    API_BASE.to_string()
}

fn client() -> Client {
    Client::new()
}

pub fn auth_header() -> Option<String> {
    get_jwt_from_storage().map(|jwt| format!("Bearer {jwt}"))
}

/// Fetch from API with automatic cache: try API → cache raw JSON → parse.
/// On failure → load from cache. One function, zero boilerplate.
pub async fn cached_get<T: serde::de::DeserializeOwned>(
    path: &str,
    store: &dyn crate::local_db::LocalStore,
    cache_key: &str,
) -> Result<T, String> {
    // Try API first
    if let Ok(raw) = api_get::<serde_json::Value>(path).await {
        store.set(cache_key, &raw);
        if let Ok(parsed) = serde_json::from_value::<T>(raw) {
            return Ok(parsed);
        }
    }
    // Fallback to cache
    store
        .get(cache_key)
        .and_then(|v| serde_json::from_value::<T>(v).ok())
        .ok_or_else(|| "Offline — no cached data".to_string())
}

pub async fn api_get<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let mut req = client().get(format!("{}{path}", api_base()));
    if let Some(auth) = auth_header() {
        req = req.header("Authorization", &auth);
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

pub async fn api_post_json<T: serde::de::DeserializeOwned>(
    path: &str,
    body: &str,
) -> Result<T, String> {
    let mut req = client()
        .post(format!("{}{path}", api_base()))
        .header("Content-Type", "application/json")
        .body(body.to_string());
    if let Some(auth) = auth_header() {
        req = req.header("Authorization", &auth);
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(msg);
    }
    resp.json().await.map_err(|e| e.to_string())
}

pub async fn api_patch_json(path: &str, body: &serde_json::Value) -> Result<(), String> {
    let mut req = client()
        .patch(format!("{}{path}", api_base()))
        .header("Content-Type", "application/json")
        .body(body.to_string());
    if let Some(auth) = auth_header() {
        req = req.header("Authorization", &auth);
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(msg);
    }
    Ok(())
}

pub async fn api_post_raw(path: &str) -> Result<serde_json::Value, String> {
    let mut req = client().post(format!("{}{path}", api_base()));
    if let Some(auth) = auth_header() {
        req = req.header("Authorization", &auth);
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(msg);
    }
    resp.json().await.map_err(|e| e.to_string())
}

pub async fn api_delete(path: &str) -> Result<(), String> {
    let mut req = client().delete(format!("{}{path}", api_base()));
    if let Some(auth) = auth_header() {
        req = req.header("Authorization", &auth);
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(msg);
    }
    Ok(())
}

#[allow(dead_code)]
pub async fn api_get_text(path: &str) -> Result<String, String> {
    let mut req = client().get(format!("{}{path}", api_base()));
    if let Some(auth) = auth_header() {
        req = req.header("Authorization", &auth);
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.text().await.map_err(|e| e.to_string())
}

// --- Data fetchers ---

pub async fn place_order(body: String) -> Result<(String, Option<String>), String> {
    let result: serde_json::Value = api_post_json("/orders", &body).await?;
    let order_id = result["order"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let checkout_url = result["checkout_url"].as_str().map(|s| s.to_string());
    Ok((order_id, checkout_url))
}

pub async fn fetch_my_orders() -> Result<Vec<serde_json::Value>, String> {
    api_get("/my/orders").await
}

pub async fn assign_courier(order_id: String) -> Result<serde_json::Value, String> {
    api_post_raw(&format!("/orders/{order_id}/assign")).await
}

pub async fn transition_order(order_id: String, status: String) -> Result<(), String> {
    api_patch_json(
        &format!("/orders/{order_id}/status"),
        &serde_json::json!({ "status": status }),
    )
    .await
}

// --- Admin ---

pub async fn toggle_user_blocked(user_id: &str, blocked: bool) -> Result<(), String> {
    api_patch_json(
        &format!("/admin/users/{user_id}/block"),
        &serde_json::json!({ "blocked": blocked }),
    )
    .await
}

pub async fn resolve_dispute(
    dispute_id: &str,
    status: &str,
    resolution: Option<&str>,
) -> Result<(), String> {
    api_patch_json(
        &format!("/admin/disputes/{dispute_id}/resolve"),
        &serde_json::json!({ "status": status, "resolution": resolution }),
    )
    .await
}

#[allow(dead_code)]
pub async fn create_dispute(order_id: &str, reason: &str) -> Result<serde_json::Value, String> {
    api_post_json(
        &format!("/orders/{order_id}/dispute"),
        &serde_json::json!({ "reason": reason }).to_string(),
    )
    .await
}

// --- My restaurants (owner) ---

pub async fn fetch_my_restaurants() -> Result<Vec<serde_json::Value>, String> {
    api_get("/my/restaurants").await
}

pub async fn fetch_restaurant_detail(id: &str) -> Result<serde_json::Value, String> {
    api_get(&format!("/restaurants/{id}")).await
}

pub async fn create_restaurant(body: &serde_json::Value) -> Result<serde_json::Value, String> {
    api_post_json("/restaurants", &body.to_string()).await
}

pub async fn update_restaurant(id: &str, body: &serde_json::Value) -> Result<(), String> {
    api_patch_json(&format!("/restaurants/{id}"), body).await
}

pub async fn toggle_restaurant_active(id: &str, active: bool) -> Result<(), String> {
    api_patch_json(
        &format!("/restaurants/{id}/active"),
        &serde_json::json!({ "active": active }),
    )
    .await
}

pub async fn add_menu_item(restaurant_id: &str, body: &serde_json::Value) -> Result<(), String> {
    let _: serde_json::Value = api_post_json(
        &format!("/restaurants/{restaurant_id}/menu"),
        &body.to_string(),
    )
    .await?;
    Ok(())
}

pub async fn delete_menu_item(item_id: &str) -> Result<(), String> {
    api_delete(&format!("/menu-items/{item_id}")).await
}

// --- Courier ---

pub async fn register_courier(body: &serde_json::Value) -> Result<serde_json::Value, String> {
    api_post_json("/couriers", &body.to_string()).await
}

// --- Config ---

pub async fn fetch_config() -> Result<serde_json::Value, String> {
    api_get("/config").await
}

// --- Helpers ---

pub fn cart_total(items: &[CartItem]) -> Money {
    items
        .iter()
        .map(|i| i.price * Decimal::from(i.quantity))
        .fold(Money::zero(), |a, b| a + b)
}
