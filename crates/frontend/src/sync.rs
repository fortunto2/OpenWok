#![allow(non_snake_case)]

//! Sync engine: pull (API → local cache), push (outbox → API).
//! WASM: JSON in localStorage. Native: structured SQL in rusqlite.

use crate::api::{api_get, api_patch_json};
use crate::local_db;

/// Pull courier deliveries and profile from API into local cache.
pub async fn pull_deliveries() {
    if let Ok(deliveries) = api_get::<serde_json::Value>("/my/deliveries").await {
        // WASM: store as JSON blob
        local_db::set("deliveries", &deliveries);

        // Native: also upsert into SQL tables
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(arr) = deliveries.as_array() {
            local_db::upsert_orders(arr);
        }
    }
    if let Ok(courier) = api_get::<serde_json::Value>("/couriers/me").await {
        local_db::set("courier_profile", &courier);

        #[cfg(not(target_arch = "wasm32"))]
        local_db::upsert_courier(&courier);
    }
}

/// Push pending offline actions to API. Returns number of actions pushed.
pub async fn push_pending() -> usize {
    // Native: use SQL-backed outbox
    #[cfg(not(target_arch = "wasm32"))]
    {
        let actions = local_db::drain_pending();
        if actions.is_empty() {
            return 0;
        }
        let mut pushed = 0;
        for (id, action, payload) in &actions {
            let success = match action.as_str() {
                "mark_delivered" => {
                    if let Some(order_id) = payload["order_id"].as_str() {
                        api_patch_json(
                            &format!("/orders/{order_id}/status"),
                            &serde_json::json!({"status": "Delivered"}),
                        )
                        .await
                        .is_ok()
                    } else {
                        true
                    }
                }
                _ => true,
            };
            if success {
                local_db::remove_pending(*id);
                pushed += 1;
            }
        }
        pushed
    }

    // WASM: use JSON-backed outbox
    #[cfg(target_arch = "wasm32")]
    {
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
                        true
                    }
                }
                _ => true,
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
}

/// Queue an offline action for later sync.
pub fn queue_action(action: &str, payload: serde_json::Value) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        local_db::queue_pending(action, &payload);
    }

    #[cfg(target_arch = "wasm32")]
    {
        let mut actions = local_db::get("pending_actions")
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();

        actions.push(serde_json::json!({
            "action": action,
            "payload": payload,
        }));

        local_db::set("pending_actions", &serde_json::Value::Array(actions));
    }
}

/// Get count of pending offline actions.
pub fn pending_count() -> usize {
    #[cfg(not(target_arch = "wasm32"))]
    {
        local_db::pending_action_count()
    }

    #[cfg(target_arch = "wasm32")]
    {
        local_db::get("pending_actions")
            .and_then(|v| v.as_array().map(|a| a.len()))
            .unwrap_or(0)
    }
}
