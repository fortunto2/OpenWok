pub mod db;
pub mod payments;
pub mod sqlite_repo;
pub mod state;
mod stripe;
mod ws;

use std::sync::Arc;

use axum::Router;
use axum::http::{
    HeaderValue, Method,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use axum::routing::{any, post};
pub use openwok_handlers::auth::JwtConfig;
pub use sqlite_repo::SqliteRepo;
pub use state::AppState;
use superduperai_auth::{AuthClient, AuthConfig, AuthError};
use tower_http::cors::CorsLayer;
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

pub async fn jwt_config_from_env() -> Result<JwtConfig, AuthError> {
    AuthClient::from_config(AuthConfig::from_server_env("openwok")?)
        .await
        .map(Arc::new)
}

pub fn router_with_jwt_config(state: AppState, jwt_config: JwtConfig) -> Router {
    let (api_handlers, openapi) =
        openwok_handlers::api_routes_with_openapi::<SqliteRepo>(ApiDoc::openapi());

    let api_handlers = api_handlers.with_state(state.repo.clone());

    let payment_routes = Router::new()
        .route("/orders", post(payments::create_order_with_payment))
        .route("/webhooks/stripe", post(payments::stripe_webhook))
        .with_state(state.clone());

    let order_events_tx = state.order_events.clone();

    let ws_route = Router::new()
        .route("/ws/orders/{id}", any(ws::order_updates))
        .with_state(state);

    let api = Router::new()
        .merge(payment_routes)
        .merge(api_handlers)
        .merge(ws_route)
        .layer(axum::Extension(jwt_config))
        .layer(axum::Extension(order_events_tx));

    let cors = CorsLayer::new()
        .allow_origin([
            "https://openwok.nameless-sunset-8f24.workers.dev"
                .parse::<HeaderValue>()
                .unwrap(),
            "https://openwok.superduperai.co"
                .parse::<HeaderValue>()
                .unwrap(),
            "http://localhost:8080".parse::<HeaderValue>().unwrap(),
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),
            "http://localhost:3030".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]);

    Router::new()
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", openapi))
        .nest("/api", api)
        .layer(cors)
}
