#![allow(non_snake_case)]

//! Sync engine: pull (API → local cache), push (outbox → API).

use crate::api::{api_get, api_patch_json};
use crate::local_db;

/// Pull courier deliveries and profile from API into local cache.
pub async fn pull_deliveries() {
    if let Ok(deliveries) = api_get::<serde_json::Value>("/my/deliveries").await {
        local_db::set("deliveries", &deliveries);
    }
    if let Ok(courier) = api_get::<serde_json::Value>("/couriers/me").await {
        local_db::set("courier_profile", &courier);
    }
}

/// Push pending offline actions to API. Returns number of actions pushed.
pub async fn push_pending() -> usize {
    let Some(actions) = local_db::get("pending_actions") else {
        return 0;
    };
    let Some(arr) = actions.as_array() else {
        return 0;
    };
    if arr.is_empty() {
        return 0;
    }

    let mut new_pending = Vec::new();
    let mut pushed = 0;

    for action in arr {
        let action_type = action["action"].as_str().unwrap_or("");
        let payload = &action["payload"];

        let success = match action_type {
            "mark_delivered" => {
                if let Some(order_id) = payload["order_id"].as_str() {
                    api_patch_json(
                        &format!("/orders/{order_id}/status"),
                        &serde_json::json!({"status": "Delivered"}),
                    )
                    .await
                    .is_ok()
                } else {
                    true // malformed, discard
                }
            }
            _ => true, // unknown action, discard
        };

        if success {
            pushed += 1;
        } else {
            new_pending.push(action.clone());
        }
    }

    if new_pending.is_empty() {
        local_db::delete("pending_actions");
    } else {
        local_db::set("pending_actions", &serde_json::Value::Array(new_pending));
    }

    pushed
}

/// Queue an offline action for later sync.
pub fn queue_action(action: &str, payload: serde_json::Value) {
    let mut actions = local_db::get("pending_actions")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();

    actions.push(serde_json::json!({
        "action": action,
        "payload": payload,
    }));

    local_db::set("pending_actions", &serde_json::Value::Array(actions));
}

/// Get count of pending offline actions.
pub fn pending_count() -> usize {
    local_db::get("pending_actions")
        .and_then(|v| v.as_array().map(|a| a.len()))
        .unwrap_or(0)
}
