#![allow(non_snake_case)]

mod app;
mod pages;
mod server_fns;
mod state;

#[cfg(feature = "server")]
mod db;

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    use std::sync::Arc;

    use dioxus_server::DioxusRouterExt;
    use tokio::sync::Mutex;

    let address = dioxus::cli_config::fullstack_address_or_localhost();

    // Initialize database
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "data/openwok.db".into());
    let conn = db::migrations::open(&db_path);
    db::migrations::seed_la_data(&conn);
    let repo = Arc::new(db::repo::SqliteRepo::new(Arc::new(Mutex::new(conn))));

    let router = axum::Router::new()
        .serve_dioxus_application(dioxus_server::ServeConfig::new(), app::AppRoot)
        .layer(axum::Extension(repo));

    let router = router.into_make_service();
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    println!("OpenWok listening on http://{address}");
    axum::serve(listener, router).await.unwrap();
}

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(app::AppRoot);
}
