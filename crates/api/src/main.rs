mod routes;
mod state;

use axum::{routing::get, Router};
use state::{AppState, SharedState};
use std::sync::Arc;
use tokio::sync::RwLock;

async fn health() -> &'static str {
    "ok"
}

pub fn app(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    let state: SharedState = Arc::new(RwLock::new(AppState::default()));

    let app = app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    println!("OpenWok API listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn test_state() -> SharedState {
        Arc::new(RwLock::new(AppState::default()))
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let app = app(test_state());
        let resp = app
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
