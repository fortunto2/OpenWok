#![allow(non_snake_case)]

//! Platform-abstracted local storage via LocalStore trait.
//! WASM: WebStore (localStorage). Native: SqliteStore (rusqlite + shared migrations).
//! Business logic (sync.rs) uses only the trait — zero cfg.

use serde_json::Value;

/// Pending action queued for offline sync.
#[derive(Debug, Clone)]
pub struct PendingAction {
    pub id: i64,
    pub action: String,
    pub payload: Value,
}

/// Platform-agnostic local storage interface.
/// Implementations: WebStore (WASM), SqliteStore (native).
pub trait LocalStore: Send + Sync {
    fn get(&self, key: &str) -> Option<Value>;
    fn set(&self, key: &str, value: &Value);
    #[allow(dead_code)]
    fn delete(&self, key: &str);
    fn queue_action(&self, action: &str, payload: &Value);
    fn drain_actions(&self) -> Vec<PendingAction>;
    fn remove_action(&self, id: i64);
    fn pending_count(&self) -> usize;
}

// ====== WASM: WebStore (localStorage) ======

#[cfg(target_arch = "wasm32")]
pub mod web_store {
    use super::*;

    const KEY_PREFIX: &str = "openwok_cache_";

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    pub struct WebStore;

    impl WebStore {
        pub fn new() -> Self {
            Self
        }
    }

    impl LocalStore for WebStore {
        fn get(&self, key: &str) -> Option<Value> {
            let s = storage()?;
            let raw = s.get_item(&format!("{KEY_PREFIX}{key}")).ok()??;
            serde_json::from_str(&raw).ok()
        }

        fn set(&self, key: &str, value: &Value) {
            if let Some(s) = storage() {
                let json = serde_json::to_string(value).unwrap_or_default();
                let _ = s.set_item(&format!("{KEY_PREFIX}{key}"), &json);
            }
        }

        fn delete(&self, key: &str) {
            if let Some(s) = storage() {
                let _ = s.remove_item(&format!("{KEY_PREFIX}{key}"));
            }
        }

        fn queue_action(&self, action: &str, payload: &Value) {
            let mut actions = self
                .get("pending_actions")
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_default();
            actions.push(serde_json::json!({
                "id": actions.len() as i64,
                "action": action,
                "payload": payload,
            }));
            self.set("pending_actions", &Value::Array(actions));
        }

        fn drain_actions(&self) -> Vec<PendingAction> {
            self.get("pending_actions")
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_default()
                .iter()
                .map(|a| PendingAction {
                    id: a["id"].as_i64().unwrap_or(0),
                    action: a["action"].as_str().unwrap_or("").to_string(),
                    payload: a["payload"].clone(),
                })
                .collect()
        }

        fn remove_action(&self, id: i64) {
            let actions = self
                .get("pending_actions")
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_default()
                .into_iter()
                .filter(|a| a["id"].as_i64() != Some(id))
                .collect::<Vec<_>>();
            if actions.is_empty() {
                self.delete("pending_actions");
            } else {
                self.set("pending_actions", &Value::Array(actions));
            }
        }

        fn pending_count(&self) -> usize {
            self.get("pending_actions")
                .and_then(|v| v.as_array().map(|a| a.len()))
                .unwrap_or(0)
        }
    }
}

// ====== Native: SqliteStore (rusqlite + shared migrations) ======

#[cfg(not(target_arch = "wasm32"))]
pub mod sqlite_store {
    use super::*;
    use rusqlite::params;
    use std::sync::Mutex;

    /// Schema from shared server migrations (single source of truth).
    const MIGRATION_INIT: &str = include_str!("../../../migrations/0001_init.sql");

    /// Extra local-only tables (outbox + sync state).
    const LOCAL_EXTRA: &str = "
CREATE TABLE IF NOT EXISTS pending_actions (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    action     TEXT NOT NULL,
    payload    TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS sync_state (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

    pub struct SqliteStore {
        conn: Mutex<rusqlite::Connection>,
    }

    impl SqliteStore {
        pub fn new() -> Self {
            let dir = dirs::data_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("co.superduperai.openwok");
            let _ = std::fs::create_dir_all(&dir);
            let db =
                rusqlite::Connection::open(dir.join("local.db")).expect("Failed to open local DB");
            db.execute_batch(MIGRATION_INIT)
                .expect("Failed to run init migration");
            db.execute_batch(LOCAL_EXTRA)
                .expect("Failed to create local tables");
            Self {
                conn: Mutex::new(db),
            }
        }
    }

    impl LocalStore for SqliteStore {
        fn get(&self, key: &str) -> Option<Value> {
            let db = self.conn.lock().ok()?;
            let mut stmt = db
                .prepare("SELECT value FROM sync_state WHERE key = ?1")
                .ok()?;
            let raw: String = stmt.query_row([key], |row| row.get(0)).ok()?;
            serde_json::from_str(&raw).ok()
        }

        fn set(&self, key: &str, value: &Value) {
            if let Ok(db) = self.conn.lock() {
                let json = serde_json::to_string(value).unwrap_or_default();
                let _ = db.execute(
                    "INSERT OR REPLACE INTO sync_state (key, value) VALUES (?1, ?2)",
                    params![key, json],
                );
            }
        }

        fn delete(&self, key: &str) {
            if let Ok(db) = self.conn.lock() {
                let _ = db.execute("DELETE FROM sync_state WHERE key = ?1", [key]);
            }
        }

        fn queue_action(&self, action: &str, payload: &Value) {
            if let Ok(db) = self.conn.lock() {
                let now = chrono::Utc::now().to_rfc3339();
                let _ = db.execute(
                    "INSERT INTO pending_actions (action, payload, created_at) VALUES (?1, ?2, ?3)",
                    params![action, payload.to_string(), now],
                );
            }
        }

        fn drain_actions(&self) -> Vec<PendingAction> {
            let Ok(db) = self.conn.lock() else {
                return Vec::new();
            };
            let Ok(mut stmt) =
                db.prepare("SELECT id, action, payload FROM pending_actions ORDER BY id")
            else {
                return Vec::new();
            };
            stmt.query_map([], |row| {
                let id: i64 = row.get(0)?;
                let action: String = row.get(1)?;
                let payload_str: String = row.get(2)?;
                let payload = serde_json::from_str(&payload_str).unwrap_or(Value::Null);
                Ok(PendingAction {
                    id,
                    action,
                    payload,
                })
            })
            .ok()
            .map(|r| r.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
        }

        fn remove_action(&self, id: i64) {
            if let Ok(db) = self.conn.lock() {
                let _ = db.execute("DELETE FROM pending_actions WHERE id = ?1", [id]);
            }
        }

        fn pending_count(&self) -> usize {
            let Ok(db) = self.conn.lock() else { return 0 };
            db.query_row("SELECT COUNT(*) FROM pending_actions", [], |row| {
                row.get::<_, i64>(0)
            })
            .unwrap_or(0) as usize
        }
    }

    /// Upsert orders into structured SQL tables (beyond key-value).
    #[allow(dead_code)]
    impl SqliteStore {
        pub fn upsert_orders(&self, orders: &[Value]) {
            let Ok(db) = self.conn.lock() else { return };
            for order in orders {
                let _ = db.execute(
                    "INSERT OR REPLACE INTO orders (id, restaurant_id, courier_id, customer_address, zone_id, status, food_total, delivery_fee, tip, federal_fee, local_ops_fee, processing_fee, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                    params![
                        order["id"].as_str().unwrap_or(""),
                        order["restaurant_id"].as_str().unwrap_or(""),
                        order["courier_id"].as_str(),
                        order["customer_address"].as_str().unwrap_or(""),
                        order["zone_id"].as_str().unwrap_or(""),
                        order["status"].as_str().unwrap_or("Created"),
                        order["pricing"]["food_total"].as_str().unwrap_or("0"),
                        order["pricing"]["delivery_fee"].as_str().unwrap_or("0"),
                        order["pricing"]["tip"].as_str().unwrap_or("0"),
                        order["pricing"]["federal_fee"].as_str().unwrap_or("0"),
                        order["pricing"]["local_ops_fee"].as_str().unwrap_or("0"),
                        order["pricing"]["processing_fee"].as_str().unwrap_or("0"),
                        order["created_at"].as_str().unwrap_or(""),
                        order["updated_at"].as_str().unwrap_or(""),
                    ],
                );
            }
        }

        pub fn upsert_courier(&self, courier: &Value) {
            let Ok(db) = self.conn.lock() else { return };
            let _ = db.execute(
                "INSERT OR REPLACE INTO couriers (id, name, kind, zone_id, available, user_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    courier["id"].as_str().unwrap_or(""),
                    courier["name"].as_str().unwrap_or(""),
                    courier["kind"].as_str().unwrap_or("Human"),
                    courier["zone_id"].as_str().unwrap_or(""),
                    courier["available"].as_bool().unwrap_or(false) as i32,
                    courier["user_id"].as_str(),
                ],
            );
        }
    }
}

// ====== Factory: create the right store for the platform ======

use std::sync::Arc;

pub type Store = Arc<dyn LocalStore>;

#[cfg(target_arch = "wasm32")]
pub fn create_store() -> Store {
    Arc::new(web_store::WebStore::new())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_store() -> Store {
    Arc::new(sqlite_store::SqliteStore::new())
}
