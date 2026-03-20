#![allow(non_snake_case)]

use gloo_net::http::Request;
use openwok_core::money::Money;
use openwok_core::types::Restaurant;
use rust_decimal::Decimal;

use crate::state::{CartItem, get_jwt_from_storage};

pub const API_BASE: &str = "/api";

pub fn auth_header() -> Option<String> {
    get_jwt_from_storage().map(|jwt| format!("Bearer {jwt}"))
}

pub async fn api_get<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let mut req = Request::get(&format!("{API_BASE}{path}"));
    if let Some(auth) = auth_header() {
        req = req.header("Authorization", &auth);
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

pub async fn api_post_json<T: serde::de::DeserializeOwned>(
    path: &str,
    body: &str,
) -> Result<T, String> {
    let mut req =
        Request::post(&format!("{API_BASE}{path}")).header("Content-Type", "application/json");
    if let Some(auth) = auth_header() {
        req = req.header("Authorization", &auth);
    }
    let resp = req
        .body(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(msg);
    }
    resp.json().await.map_err(|e| e.to_string())
}

pub async fn api_patch_json(path: &str, body: &serde_json::Value) -> Result<(), String> {
    let mut req =
        Request::patch(&format!("{API_BASE}{path}")).header("Content-Type", "application/json");
    if let Some(auth) = auth_header() {
        req = req.header("Authorization", &auth);
    }
    let resp = req
        .body(body.to_string())
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(msg);
    }
    Ok(())
}

// --- Data fetchers ---

pub async fn fetch_restaurants() -> Result<Vec<Restaurant>, String> {
    api_get("/restaurants").await
}

pub async fn fetch_restaurant(id: String) -> Result<Restaurant, String> {
    api_get(&format!("/restaurants/{id}")).await
}

/// Returns (order_id, checkout_url option)
pub async fn place_order(body: String) -> Result<(String, Option<String>), String> {
    let result: serde_json::Value = api_post_json("/orders", &body).await?;
    let order_id = result["order"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let checkout_url = result["checkout_url"].as_str().map(|s| s.to_string());
    Ok((order_id, checkout_url))
}

pub async fn fetch_order(id: String) -> Result<serde_json::Value, String> {
    api_get(&format!("/orders/{id}")).await
}

pub async fn fetch_dashboard() -> Result<serde_json::Value, String> {
    let restaurants: Vec<serde_json::Value> = api_get("/restaurants").await?;
    let couriers: Vec<serde_json::Value> = api_get("/couriers").await?;
    Ok(serde_json::json!({
        "restaurant_count": restaurants.len(),
        "couriers_online": couriers.len(),
        "restaurants": restaurants,
        "couriers": couriers,
    }))
}

pub async fn fetch_economics() -> Result<serde_json::Value, String> {
    api_get("/public/economics").await
}

pub async fn fetch_admin_metrics() -> Result<serde_json::Value, String> {
    api_get("/admin/metrics").await
}

pub async fn fetch_all_orders() -> Result<Vec<serde_json::Value>, String> {
    api_get("/orders").await
}

pub async fn assign_courier(order_id: String) -> Result<serde_json::Value, String> {
    let mut req = Request::post(&format!("{API_BASE}/orders/{order_id}/assign"));
    if let Some(auth) = auth_header() {
        req = req.header("Authorization", &auth);
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(msg);
    }
    resp.json().await.map_err(|e| e.to_string())
}

pub async fn transition_order(order_id: String, status: String) -> Result<(), String> {
    api_patch_json(
        &format!("/orders/{order_id}/status"),
        &serde_json::json!({ "status": status }),
    )
    .await
}

// --- Admin ---

pub async fn fetch_admin_users() -> Result<Vec<serde_json::Value>, String> {
    api_get("/admin/users").await
}

pub async fn toggle_user_blocked(user_id: &str, blocked: bool) -> Result<(), String> {
    api_patch_json(
        &format!("/admin/users/{user_id}/block"),
        &serde_json::json!({ "blocked": blocked }),
    )
    .await
}

pub async fn fetch_admin_disputes() -> Result<Vec<serde_json::Value>, String> {
    api_get("/admin/disputes").await
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

// --- Helpers ---

pub fn cart_total(items: &[CartItem]) -> Money {
    items
        .iter()
        .map(|i| i.price * Decimal::from(i.quantity))
        .fold(Money::zero(), |a, b| a + b)
}
