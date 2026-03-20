pub mod db;
pub mod payments;
pub mod sqlite_repo;
pub mod state;
pub mod stripe;
mod ws;

use std::sync::Arc;

use axum::Router;
use axum::routing::{any, post};
use openwok_handlers::auth::JwtConfig;
use sqlite_repo::SqliteRepo;
use state::AppState;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "OpenWok API",
        description = "Fair food delivery — $1 federal fee, 100% transparency",
        version = "0.1.0"
    ),
    servers((url = "/api")),
    components(schemas(
        openwok_core::types::Restaurant,
        openwok_core::types::MenuItem,
        openwok_core::types::Courier,
        openwok_core::types::CourierKind,
        openwok_core::types::Zone,
        openwok_core::types::User,
        openwok_core::types::UserRole,
        openwok_core::types::Payment,
        openwok_core::types::PaymentStatus,
        openwok_core::order::Order,
        openwok_core::order::OrderItem,
        openwok_core::order::OrderStatus,
        openwok_core::pricing::PricingBreakdown,
        openwok_core::money::Money,
    ))
)]
struct ApiDoc;

pub fn app(state: AppState) -> Router {
    let jwt_config = JwtConfig {
        secret: std::env::var("SUPABASE_JWT_SECRET")
            .unwrap_or_else(|_| "super-secret-jwt-token-for-testing-only".into()),
        issuer: std::env::var("SUPABASE_JWT_ISSUER").ok(),
    };

    let (api_handlers, openapi) =
        openwok_handlers::api_routes_with_openapi::<SqliteRepo>(ApiDoc::openapi());

    // Shared handlers use Arc<SqliteRepo> state + JwtConfig extension
    let api_handlers = api_handlers
        .with_state(state.repo.clone())
        .layer(axum::Extension(jwt_config));

    // Payment routes use full AppState (need StripeClient + repo)
    let payment_routes = Router::new()
        .route("/orders", post(payments::create_order_with_payment))
        .route("/webhooks/stripe", post(payments::stripe_webhook))
        .with_state(state.clone());

    // WS route uses full AppState (needs broadcast channel)
    let ws_route = Router::new()
        .route("/ws/orders/{id}", any(ws::order_updates))
        .with_state(state);

    let api = Router::new()
        .merge(payment_routes) // Payment routes override generic order create
        .merge(api_handlers)
        .merge(ws_route);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", openapi))
        .nest("/api", api)
        .layer(cors)
}

#[tokio::main]
async fn main() {
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "data/openwok.db".into());
    let conn = db::open(&db_path);
    db::seed_la_data(&conn);
    let repo = Arc::new(SqliteRepo::new(Arc::new(Mutex::new(conn))));
    let state = AppState::new(repo);
    let app = app(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3030".into());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("OpenWok API listening on http://localhost:{port}/api");
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn test_state() -> AppState {
        let conn = db::open(":memory:");
        let repo = Arc::new(SqliteRepo::new(Arc::new(Mutex::new(conn))));
        AppState::new(repo)
    }

    fn seeded_state() -> AppState {
        let conn = db::open(":memory:");
        db::seed_la_data(&conn);
        let repo = Arc::new(SqliteRepo::new(Arc::new(Mutex::new(conn))));
        AppState::new(repo)
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let app = app(test_state());
        let resp = app
            .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn list_restaurants_returns_seeded() {
        let app = app(seeded_state());
        let resp = app
            .oneshot(
                Request::get("/api/restaurants")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let restaurants: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert_eq!(restaurants.len(), 18);
    }

    #[tokio::test]
    async fn full_order_flow() {
        let state = seeded_state();
        let app = app(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let base = format!("http://{addr}/api");
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
        assert_eq!(restaurants.len(), 18);
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
        let result: serde_json::Value = resp.json().await.unwrap();

        // Response is { order, checkout_url, payment_id }
        let order = &result["order"];
        assert!(result["payment_id"].is_string(), "payment record created");

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
