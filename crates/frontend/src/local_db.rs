#![allow(non_snake_case)]

//! Platform-abstracted local cache.
//! WASM: localStorage (web_sys::Storage) — key-value JSON.
//! Native: rusqlite — same schema as backend D1, shared migrations.

use serde_json::Value;

// ====== WASM: localStorage ======

#[cfg(target_arch = "wasm32")]
const KEY_PREFIX: &str = "openwok_cache_";

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

#[cfg(target_arch = "wasm32")]
pub fn init() {
    // localStorage needs no init
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

// ====== Native: rusqlite with shared schema ======

#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;

#[cfg(not(target_arch = "wasm32"))]
static DB: OnceLock<std::sync::Mutex<rusqlite::Connection>> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
fn db_path() -> std::path::PathBuf {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("co.superduperai.openwok");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("local.db")
}

#[cfg(not(target_arch = "wasm32"))]
fn conn() -> &'static std::sync::Mutex<rusqlite::Connection> {
    DB.get_or_init(|| {
        let db = rusqlite::Connection::open(db_path()).expect("Failed to open local SQLite");
        // Run local schema — subset of server migrations
        db.execute_batch(LOCAL_SCHEMA)
            .expect("Failed to init local schema");
        std::sync::Mutex::new(db)
    })
}

/// Local schema: same structure as server, subset of tables.
/// Compatible with server migrations (same column names/types).
#[cfg(not(target_arch = "wasm32"))]
const LOCAL_SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS couriers (
    id        TEXT PRIMARY KEY,
    name      TEXT NOT NULL,
    kind      TEXT NOT NULL DEFAULT 'Human',
    zone_id   TEXT NOT NULL,
    available INTEGER NOT NULL DEFAULT 1,
    user_id   TEXT
);

CREATE TABLE IF NOT EXISTS orders (
    id               TEXT PRIMARY KEY,
    restaurant_id    TEXT NOT NULL,
    courier_id       TEXT,
    customer_address TEXT NOT NULL,
    zone_id          TEXT NOT NULL,
    status           TEXT NOT NULL DEFAULT 'Created',
    food_total       TEXT NOT NULL,
    delivery_fee     TEXT NOT NULL,
    tip              TEXT NOT NULL,
    federal_fee      TEXT NOT NULL,
    local_ops_fee    TEXT NOT NULL,
    processing_fee   TEXT NOT NULL,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS order_items (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id     TEXT NOT NULL,
    menu_item_id TEXT NOT NULL,
    name         TEXT NOT NULL,
    quantity     INTEGER NOT NULL,
    unit_price   TEXT NOT NULL
);

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

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn init() {
    let _ = conn(); // triggers OnceLock init + schema
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get(key: &str) -> Option<Value> {
    let db = conn().lock().ok()?;
    let mut stmt = db
        .prepare("SELECT value FROM sync_state WHERE key = ?1")
        .ok()?;
    let raw: String = stmt.query_row([key], |row| row.get(0)).ok()?;
    serde_json::from_str(&raw).ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set(key: &str, value: &Value) {
    if let Ok(db) = conn().lock() {
        let json = serde_json::to_string(value).unwrap_or_default();
        let _ = db.execute(
            "INSERT OR REPLACE INTO sync_state (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, json],
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn delete(key: &str) {
    if let Ok(db) = conn().lock() {
        let _ = db.execute("DELETE FROM sync_state WHERE key = ?1", [key]);
    }
}

// ====== Native-only: direct SQL access for structured data ======

/// Upsert orders from API response into local SQLite (native only).
#[cfg(not(target_arch = "wasm32"))]
pub fn upsert_orders(orders: &[Value]) {
    let Ok(db) = conn().lock() else { return };
    for order in orders {
        let _ = db.execute(
            "INSERT OR REPLACE INTO orders (id, restaurant_id, courier_id, customer_address, zone_id, status, food_total, delivery_fee, tip, federal_fee, local_ops_fee, processing_fee, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            rusqlite::params![
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

/// Upsert courier profile into local SQLite (native only).
#[cfg(not(target_arch = "wasm32"))]
pub fn upsert_courier(courier: &Value) {
    let Ok(db) = conn().lock() else { return };
    let _ = db.execute(
        "INSERT OR REPLACE INTO couriers (id, name, kind, zone_id, available, user_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            courier["id"].as_str().unwrap_or(""),
            courier["name"].as_str().unwrap_or(""),
            courier["kind"].as_str().unwrap_or("Human"),
            courier["zone_id"].as_str().unwrap_or(""),
            courier["available"].as_bool().unwrap_or(false) as i32,
            courier["user_id"].as_str(),
        ],
    );
}

/// Queue a pending action (native only — uses SQL table).
#[cfg(not(target_arch = "wasm32"))]
pub fn queue_pending(action: &str, payload: &Value) {
    let Ok(db) = conn().lock() else { return };
    let now = chrono::Utc::now().to_rfc3339();
    let _ = db.execute(
        "INSERT INTO pending_actions (action, payload, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![action, payload.to_string(), now],
    );
}

/// Get and drain pending actions (native only).
#[cfg(not(target_arch = "wasm32"))]
pub fn drain_pending() -> Vec<(i64, String, Value)> {
    let Ok(db) = conn().lock() else {
        return Vec::new();
    };
    let Ok(mut stmt) = db.prepare("SELECT id, action, payload FROM pending_actions ORDER BY id")
    else {
        return Vec::new();
    };
    let rows = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let action: String = row.get(1)?;
            let payload_str: String = row.get(2)?;
            let payload = serde_json::from_str(&payload_str).unwrap_or(Value::Null);
            Ok((id, action, payload))
        })
        .ok();
    let result: Vec<_> = rows
        .map(|r| r.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();
    result
}

/// Remove a pending action by id (native only).
#[cfg(not(target_arch = "wasm32"))]
pub fn remove_pending(id: i64) {
    let Ok(db) = conn().lock() else { return };
    let _ = db.execute("DELETE FROM pending_actions WHERE id = ?1", [id]);
}

/// Count pending actions (native only).
#[cfg(not(target_arch = "wasm32"))]
pub fn pending_action_count() -> usize {
    let Ok(db) = conn().lock() else { return 0 };
    db.query_row("SELECT COUNT(*) FROM pending_actions", [], |row| {
        row.get::<_, i64>(0)
    })
    .unwrap_or(0) as usize
}
