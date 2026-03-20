#![allow(non_snake_case)]

use openwok_core::money::Money;
use openwok_core::types::Restaurant;
use reqwest::Client;
use rust_decimal::Decimal;

use crate::config::API_BASE;
use crate::state::CartItem;
use crate::storage::load_jwt;

fn client() -> Client {
    Client::new()
}

fn auth_header() -> Option<String> {
    load_jwt().map(|jwt| format!("Bearer {jwt}"))
}

pub async fn api_get<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let mut req = client().get(format!("{API_BASE}{path}"));
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
        .post(format!("{API_BASE}{path}"))
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
        .patch(format!("{API_BASE}{path}"))
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

// --- Data fetchers ---

pub async fn fetch_restaurants() -> Result<Vec<Restaurant>, String> {
    api_get("/restaurants").await
}

pub async fn fetch_restaurant(id: &str) -> Result<Restaurant, String> {
    api_get(&format!("/restaurants/{id}")).await
}

pub async fn place_order(body: String) -> Result<(String, Option<String>), String> {
    let result: serde_json::Value = api_post_json("/orders", &body).await?;
    let order_id = result["order"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let checkout_url = result["checkout_url"].as_str().map(|s| s.to_string());
    Ok((order_id, checkout_url))
}

pub async fn fetch_order(id: &str) -> Result<serde_json::Value, String> {
    api_get(&format!("/orders/{id}")).await
}

pub async fn fetch_courier_me() -> Result<serde_json::Value, String> {
    api_get("/couriers/me").await
}

pub async fn fetch_my_deliveries() -> Result<Vec<serde_json::Value>, String> {
    api_get("/my/deliveries").await
}

pub async fn transition_order(order_id: &str, status: &str) -> Result<(), String> {
    api_patch_json(
        &format!("/orders/{order_id}/status"),
        &serde_json::json!({ "status": status }),
    )
    .await
}

pub async fn toggle_courier_availability(courier_id: &str, available: bool) -> Result<(), String> {
    api_patch_json(
        &format!("/couriers/{courier_id}/available"),
        &serde_json::json!({ "available": available }),
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
