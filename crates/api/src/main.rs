mod routes;
mod state;

use axum::Router;
use axum::routing::{any, get, patch, post};
use state::AppState;

async fn health() -> &'static str {
    "ok"
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/restaurants",
            get(routes::restaurants::list).post(routes::restaurants::create),
        )
        .route("/restaurants/{id}", get(routes::restaurants::get))
        .route(
            "/orders",
            get(routes::orders::list).post(routes::orders::create),
        )
        .route("/orders/{id}", get(routes::orders::get))
        .route("/orders/{id}/status", patch(routes::orders::transition))
        .route(
            "/orders/{id}/assign",
            post(routes::couriers::assign_to_order),
        )
        .route(
            "/couriers",
            get(routes::couriers::list).post(routes::couriers::create),
        )
        .route(
            "/couriers/{id}/available",
            patch(routes::couriers::toggle_available),
        )
        .route("/ws/orders/{id}", any(routes::ws::order_updates))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    let state = AppState::new();
    {
        let mut data = state.data.write().await;
        routes::restaurants::seed_restaurants(&mut data);
    }

    let app = app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("OpenWok API listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn test_state() -> AppState {
        AppState::new()
    }

    async fn seeded_state() -> AppState {
        let state = AppState::new();
        let mut data = state.data.write().await;
        routes::restaurants::seed_restaurants(&mut data);
        drop(data);
        state
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
        let app = app(seeded_state().await);
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

    /// Full integration test: create restaurant → create order → see pricing →
    /// confirm → assign courier → transition to delivered.
    #[tokio::test]
    async fn full_order_flow() {
        let state = seeded_state().await;
        let app = app(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let base = format!("http://{addr}");
        let client = reqwest::Client::new();

        // 1. List restaurants — get first one
        let restaurants: Vec<serde_json::Value> = client
            .get(format!("{base}/restaurants"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(restaurants.len(), 3);
        let restaurant = &restaurants[0];
        let restaurant_id = restaurant["id"].as_str().unwrap();
        let zone_id = restaurant["zone_id"].as_str().unwrap();
        let menu_item = &restaurant["menu"][0];

        // 2. Create order
        let order_body = serde_json::json!({
            "restaurant_id": restaurant_id,
            "items": [{
                "menu_item_id": menu_item["id"],
                "name": menu_item["name"],
                "quantity": 2,
                "unit_price": menu_item["price"],
            }],
            "customer_address": "456 Oak Ave, LA",
            "zone_id": zone_id,
            "delivery_fee": "5.00",
            "tip": "3.00",
            "local_ops_fee": "2.50",
        });

        let resp = client
            .post(format!("{base}/orders"))
            .json(&order_body)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 201);
        let order: serde_json::Value = resp.json().await.unwrap();

        // Verify pricing breakdown has 6 fields
        let pricing = &order["pricing"];
        assert!(pricing["food_total"].is_string());
        assert!(pricing["delivery_fee"].is_string());
        assert!(pricing["tip"].is_string());
        assert!(pricing["federal_fee"].is_string());
        assert!(pricing["local_ops_fee"].is_string());
        assert!(pricing["processing_fee"].is_string());
        assert_eq!(pricing["federal_fee"].as_str().unwrap(), "1.00");

        let order_id = order["id"].as_str().unwrap();
        assert_eq!(order["status"].as_str().unwrap(), "Created");

        // 3. Create courier in same zone
        let courier_resp = client
            .post(format!("{base}/couriers"))
            .json(&serde_json::json!({ "name": "Alex", "zone_id": zone_id }))
            .send()
            .await
            .unwrap();
        assert_eq!(courier_resp.status(), 201);

        // 4. Confirm order
        let resp = client
            .patch(format!("{base}/orders/{order_id}/status"))
            .json(&serde_json::json!({ "status": "Confirmed" }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);

        // 5. Assign courier
        let resp = client
            .post(format!("{base}/orders/{order_id}/assign"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);

        // 6. Transition through remaining states
        for status in ["Preparing", "ReadyForPickup", "InDelivery", "Delivered"] {
            let resp = client
                .patch(format!("{base}/orders/{order_id}/status"))
                .json(&serde_json::json!({ "status": status }))
                .send()
                .await
                .unwrap();
            assert_eq!(resp.status(), 200, "failed to transition to {status}");
        }

        // 7. Verify final state
        let final_order: serde_json::Value = client
            .get(format!("{base}/orders/{order_id}"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(final_order["status"].as_str().unwrap(), "Delivered");
        assert!(final_order["courier_id"].as_str().is_some());
    }
}
