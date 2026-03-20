use std::sync::Arc;

use serde::Serialize;
use tokio::sync::broadcast;

use crate::sqlite_repo::SqliteRepo;

#[derive(Clone, Debug, Serialize)]
pub struct OrderEvent {
    pub order_id: String,
    pub status: String,
}

/// Combined state: shared SqliteRepo (for handlers crate) + broadcast channel (for WS).
#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<SqliteRepo>,
    pub order_events: broadcast::Sender<OrderEvent>,
}

impl AppState {
    pub fn new(repo: Arc<SqliteRepo>) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            repo,
            order_events: tx,
        }
    }
}

/// Allows handlers crate to extract `State<Arc<SqliteRepo>>` from AppState.
impl axum::extract::FromRef<AppState> for Arc<SqliteRepo> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone()
    }
}
