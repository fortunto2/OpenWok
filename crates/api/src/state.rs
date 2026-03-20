use rusqlite::Connection;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

#[derive(Clone, Debug, Serialize)]
pub struct OrderEvent {
    pub order_id: String,
    pub status: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub order_events: broadcast::Sender<OrderEvent>,
}

impl AppState {
    pub fn new(conn: Connection) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            db: Arc::new(Mutex::new(conn)),
            order_events: tx,
        }
    }
}
