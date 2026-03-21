use std::sync::Arc;

use openwok_api::{AppState, SqliteRepo, db, jwt_config_from_env, router_with_jwt_config};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "data/openwok.db".into());
    let conn = db::open(&db_path);
    db::seed_la_data(&conn);
    let repo = Arc::new(SqliteRepo::new(Arc::new(Mutex::new(conn))));
    let state = AppState::new(repo);
    let jwt_config = jwt_config_from_env()
        .await
        .expect("SUPABASE_URL must be set before building the API");
    let app = router_with_jwt_config(state, jwt_config);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3030".into());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("OpenWok API listening on http://localhost:{port}/api");
    axum::serve(listener, app).await.unwrap();
}
