#![allow(non_snake_case)]

//! Sync engine: pull (API → local cache), push (outbox → API).
//! Platform-agnostic — works through LocalStore trait. Zero cfg.

use crate::api::{api_get, api_patch_json};
use crate::local_db::LocalStore;

/// Pull courier deliveries and profile from API into local cache.
pub async fn pull_deliveries(store: &dyn LocalStore) {
    if let Ok(deliveries) = api_get::<serde_json::Value>("/my/deliveries").await {
        store.set("deliveries", &deliveries);
    }
    if let Ok(courier) = api_get::<serde_json::Value>("/couriers/me").await {
        store.set("courier_profile", &courier);
    }
}

/// Push pending offline actions to API. Returns number of actions pushed.
pub async fn push_pending(store: &dyn LocalStore) -> usize {
    let actions = store.drain_actions();
    if actions.is_empty() {
        return 0;
    }

    let mut pushed = 0;
    for action in &actions {
        let success = match action.action.as_str() {
            "mark_delivered" => {
                if let Some(order_id) = action.payload["order_id"].as_str() {
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
            _ => true, // unknown, discard
        };

        if success {
            store.remove_action(action.id);
            pushed += 1;
        }
    }

    pushed
}

/// Queue an offline action for later sync.
pub fn queue_action(store: &dyn LocalStore, action: &str, payload: serde_json::Value) {
    store.queue_action(action, &payload);
}

/// Get count of pending offline actions.
pub fn pending_count(store: &dyn LocalStore) -> usize {
    store.pending_count()
}
