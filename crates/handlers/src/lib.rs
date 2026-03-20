pub mod auth;
pub mod auth_handlers;
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
        .routes(routes!(restaurants::get, restaurants::update))
        .routes(routes!(restaurants::toggle_active))
        .routes(routes!(restaurants::add_menu_item))
        .routes(routes!(restaurants::update_menu_item))
        .routes(routes!(restaurants::delete_menu_item))
        .routes(routes!(restaurants::my_restaurants))
        .routes(routes!(orders::list))
        .routes(routes!(orders::create))
        .routes(routes!(orders::get))
        .routes(routes!(orders::transition))
        .routes(routes!(couriers::assign_to_order))
        .routes(routes!(couriers::list, couriers::create))
        .routes(routes!(couriers::toggle_available))
        .routes(routes!(couriers::me))
        .routes(routes!(couriers::my_deliveries))
        .routes(routes!(economics::get))
        .routes(routes!(metrics::get))
        .routes(routes!(auth_handlers::callback))
        .routes(routes!(auth_handlers::me))
        .split_for_parts();
    router
}

/// Build the shared API router and return both the router and OpenAPI spec.
/// Excludes POST /orders so the api crate can provide its own payment-aware handler.
pub fn api_routes_with_openapi<R: Repository>(
    openapi: utoipa::openapi::OpenApi,
) -> (Router<Arc<R>>, utoipa::openapi::OpenApi) {
    utoipa_axum::router::OpenApiRouter::<Arc<R>>::with_openapi(openapi)
        .route("/health", get(health))
        .routes(routes!(restaurants::list, restaurants::create))
        .routes(routes!(restaurants::get, restaurants::update))
        .routes(routes!(restaurants::toggle_active))
        .routes(routes!(restaurants::add_menu_item))
        .routes(routes!(restaurants::update_menu_item))
        .routes(routes!(restaurants::delete_menu_item))
        .routes(routes!(restaurants::my_restaurants))
        .routes(routes!(orders::list))
        .routes(routes!(orders::get))
        .routes(routes!(orders::transition))
        .routes(routes!(couriers::assign_to_order))
        .routes(routes!(couriers::list, couriers::create))
        .routes(routes!(couriers::toggle_available))
        .routes(routes!(couriers::me))
        .routes(routes!(couriers::my_deliveries))
        .routes(routes!(economics::get))
        .routes(routes!(metrics::get))
        .routes(routes!(auth_handlers::callback))
        .routes(routes!(auth_handlers::me))
        .split_for_parts()
}

async fn health() -> &'static str {
    "ok"
}
