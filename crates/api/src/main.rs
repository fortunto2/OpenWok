mod routes;
mod state;

use axum::routing::{get, patch, post};
use axum::Router;
use state::{AppState, SharedState};
use std::sync::Arc;
use tokio::sync::RwLock;

async fn health() -> &'static str {
    "ok"
}

pub fn app(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/restaurants", get(routes::restaurants::list).post(routes::restaurants::create))
        .route("/restaurants/{id}", get(routes::restaurants::get))
        .route("/orders", post(routes::orders::create))
        .route("/orders/{id}", get(routes::orders::get))
        .route("/orders/{id}/status", patch(routes::orders::transition))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    let mut initial = AppState::default();
    routes::restaurants::seed_restaurants(&mut initial);
    let state: SharedState = Arc::new(RwLock::new(initial));

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

    fn seeded_state() -> SharedState {
        let mut s = AppState::default();
        routes::restaurants::seed_restaurants(&mut s);
        Arc::new(RwLock::new(s))
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

    #[tokio::test]
    async fn list_restaurants_returns_seeded() {
        let app = app(seeded_state());
        let resp = app
            .oneshot(Request::get("/restaurants").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let restaurants: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert_eq!(restaurants.len(), 3);
    }
}
