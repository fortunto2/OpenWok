pub mod auth;
pub mod couriers;
pub mod economics;
pub mod metrics;
pub mod orders;
pub mod restaurants;

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use openwok_core::repo::Repository;
use utoipa_axum::routes;

/// Build the shared API router, generic over any Repository implementation.
/// Does NOT include WebSocket route (that's api-only).
pub fn api_routes<R: Repository>() -> Router<Arc<R>> {
    let (router, _openapi) = utoipa_axum::router::OpenApiRouter::<Arc<R>>::new()
        .route("/health", get(health))
        .routes(routes!(restaurants::list, restaurants::create))
        .routes(routes!(restaurants::get))
        .routes(routes!(orders::list, orders::create))
        .routes(routes!(orders::get))
        .routes(routes!(orders::transition))
        .routes(routes!(couriers::assign_to_order))
        .routes(routes!(couriers::list, couriers::create))
        .routes(routes!(couriers::toggle_available))
        .routes(routes!(economics::get))
        .routes(routes!(metrics::get))
        .split_for_parts();
    router
}

/// Build the shared API router and return both the router and OpenAPI spec.
pub fn api_routes_with_openapi<R: Repository>(
    openapi: utoipa::openapi::OpenApi,
) -> (Router<Arc<R>>, utoipa::openapi::OpenApi) {
    utoipa_axum::router::OpenApiRouter::<Arc<R>>::with_openapi(openapi)
        .route("/health", get(health))
        .routes(routes!(restaurants::list, restaurants::create))
        .routes(routes!(restaurants::get))
        .routes(routes!(orders::list, orders::create))
        .routes(routes!(orders::get))
        .routes(routes!(orders::transition))
        .routes(routes!(couriers::assign_to_order))
        .routes(routes!(couriers::list, couriers::create))
        .routes(routes!(couriers::toggle_available))
        .routes(routes!(economics::get))
        .routes(routes!(metrics::get))
        .split_for_parts()
}

async fn health() -> &'static str {
    "ok"
}
