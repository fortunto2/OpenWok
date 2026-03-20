use std::sync::Arc;

use serde::Serialize;
use stripe_universal::StripeClient;
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
    pub stripe_client: Option<Arc<StripeClient>>,
    pub stripe_webhook_secret: Option<String>,
}

impl AppState {
    pub fn new(repo: Arc<SqliteRepo>) -> Self {
        let (tx, _) = broadcast::channel(256);

        let stripe_client = std::env::var("STRIPE_SECRET_KEY")
            .ok()
            .map(|key| Arc::new(StripeClient::new(key)));

        let stripe_webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET").ok();

        Self {
            repo,
            order_events: tx,
            stripe_client,
            stripe_webhook_secret,
        }
    }
}

/// Allows handlers crate to extract `State<Arc<SqliteRepo>>` from AppState.
impl axum::extract::FromRef<AppState> for Arc<SqliteRepo> {
    fn from_ref(state: &AppState) -> Self {
        state.repo.clone()
    }
}
