#![allow(non_snake_case)]

mod app;
mod pages;
mod server_fns;
mod state;

#[cfg(feature = "server")]
use std::sync::Arc;

#[cfg(feature = "server")]
use dioxus::prelude::{DioxusRouterExt, ServeConfig};
#[cfg(feature = "server")]
use dioxus::server::FullstackState;
#[cfg(feature = "server")]
use openwok_api::{
    AppState as ExternalApiState, JwtConfig, SqliteRepo, db, router_with_jwt_config,
};
#[cfg(feature = "server")]
use tokio::sync::Mutex;

#[cfg(feature = "server")]
fn build_repo(db_path: &str) -> Arc<SqliteRepo> {
    let conn = db::open(db_path);
    db::seed_la_data(&conn);
    Arc::new(SqliteRepo::new(Arc::new(Mutex::new(conn))))
}

#[cfg(feature = "server")]
fn build_router(
    repo: Arc<SqliteRepo>,
    jwt_config: JwtConfig,
    serve_static_assets: bool,
) -> axum::Router {
    let api_state = ExternalApiState::new(repo.clone());
    let app_router = axum::Router::<FullstackState>::new();
    let app_router = if serve_static_assets {
        app_router.serve_dioxus_application(ServeConfig::new(), app::AppRoot)
    } else {
        app_router.serve_api_application(ServeConfig::new(), app::AppRoot)
    };

    app_router
        .merge(router_with_jwt_config(api_state, jwt_config.clone()))
        .layer(axum::Extension(jwt_config))
        .layer(axum::Extension(repo))
        .layer(tower_http::cors::CorsLayer::permissive())
}

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    let address = dioxus::cli_config::fullstack_address_or_localhost();
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "data/openwok.db".into());
    let repo = build_repo(&db_path);
    let jwt_config = openwok_api::jwt_config_from_env()
        .await
        .expect("SUPABASE_URL must be set in production");
    let router = build_router(repo, jwt_config, true);

    let router = router.into_make_service();
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    println!("OpenWok listening on http://{address}");
    axum::serve(listener, router).await.unwrap();
}

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(app::AppRoot);
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use superduperai_auth::{AuthClient, AuthConfig};
    use tower::ServiceExt;

    fn test_router() -> axum::Router {
        let repo = build_repo(":memory:");
        let jwt_config = Arc::new(AuthClient::new(
            AuthConfig::server_only("openwok", "test-secret").with_jwt_issuer("test-issuer"),
        ));

        build_router(repo, jwt_config, false)
    }

    #[tokio::test]
    async fn merged_runtime_serves_home_page() {
        let response = test_router()
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("OpenWok"));
        assert!(html.contains("Browse Restaurants"));
    }

    #[tokio::test]
    async fn merged_runtime_serves_api_health_and_seeded_restaurants() {
        let health = test_router()
            .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(health.status(), StatusCode::OK);

        let restaurants = test_router()
            .oneshot(
                Request::get("/api/restaurants")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(restaurants.status(), StatusCode::OK);

        let body = to_bytes(restaurants.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let restaurants = payload.as_array().unwrap();
        assert_eq!(restaurants.len(), 18);
        assert_eq!(restaurants[0]["name"], "Pad Thai Palace");
    }

    #[tokio::test]
    async fn merged_runtime_serves_swagger_and_openapi() {
        let docs = test_router()
            .oneshot(Request::get("/api/docs").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(docs.status(), StatusCode::SEE_OTHER);

        let openapi = test_router()
            .oneshot(
                Request::get("/api/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(openapi.status(), StatusCode::OK);

        let body = to_bytes(openapi.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["info"]["title"], "OpenWok API");
        assert_eq!(payload["servers"][0]["url"], "/api");
    }
}
