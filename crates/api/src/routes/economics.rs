use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use serde::Serialize;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct PublicEconomics {
    pub total_orders: i64,
    pub total_food_revenue: String,
    pub total_delivery_fees: String,
    pub total_federal_fees: String,
    pub total_local_ops_fees: String,
    pub total_processing_fees: String,
    pub avg_order_value: String,
}

pub async fn get(State(state): State<AppState>) -> (HeaderMap, Json<PublicEconomics>) {
    let conn = state.db.lock().await;

    let economics = conn
        .query_row(
            "SELECT
                COUNT(*) as total_orders,
                COALESCE(SUM(CAST(food_total AS REAL)), 0) as total_food_revenue,
                COALESCE(SUM(CAST(delivery_fee AS REAL)), 0) as total_delivery_fees,
                COALESCE(SUM(CAST(federal_fee AS REAL)), 0) as total_federal_fees,
                COALESCE(SUM(CAST(local_ops_fee AS REAL)), 0) as total_local_ops_fees,
                COALESCE(SUM(CAST(processing_fee AS REAL)), 0) as total_processing_fees,
                CASE WHEN COUNT(*) > 0
                    THEN (COALESCE(SUM(CAST(food_total AS REAL)), 0)
                        + COALESCE(SUM(CAST(delivery_fee AS REAL)), 0)
                        + COALESCE(SUM(CAST(federal_fee AS REAL)), 0)
                        + COALESCE(SUM(CAST(local_ops_fee AS REAL)), 0)
                        + COALESCE(SUM(CAST(processing_fee AS REAL)), 0))
                        / COUNT(*)
                    ELSE 0
                END as avg_order_value
             FROM orders",
            [],
            |row| {
                Ok(PublicEconomics {
                    total_orders: row.get(0)?,
                    total_food_revenue: format!("{:.2}", row.get::<_, f64>(1)?),
                    total_delivery_fees: format!("{:.2}", row.get::<_, f64>(2)?),
                    total_federal_fees: format!("{:.2}", row.get::<_, f64>(3)?),
                    total_local_ops_fees: format!("{:.2}", row.get::<_, f64>(4)?),
                    total_processing_fees: format!("{:.2}", row.get::<_, f64>(5)?),
                    avg_order_value: format!("{:.2}", row.get::<_, f64>(6)?),
                })
            },
        )
        .unwrap_or(PublicEconomics {
            total_orders: 0,
            total_food_revenue: "0.00".into(),
            total_delivery_fees: "0.00".into(),
            total_federal_fees: "0.00".into(),
            total_local_ops_fees: "0.00".into(),
            total_processing_fees: "0.00".into(),
            avg_order_value: "0.00".into(),
        });

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "public, max-age=300".parse().unwrap(),
    );

    (headers, Json(economics))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app;
    use crate::db;
    use crate::state::AppState;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn test_state() -> AppState {
        let conn = db::open(":memory:");
        AppState::new(conn)
    }

    fn seeded_state() -> AppState {
        let conn = db::open(":memory:");
        db::seed_la_data(&conn);
        AppState::new(conn)
    }

    #[tokio::test]
    async fn economics_empty_db() {
        let app = app(test_state());
        let resp = app
            .oneshot(
                Request::get("/api/public/economics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(data["total_orders"], 0);
        assert_eq!(data["total_food_revenue"], "0.00");
        assert_eq!(data["avg_order_value"], "0.00");
    }

    #[tokio::test]
    async fn economics_has_cache_header() {
        let app = app(test_state());
        let resp = app
            .oneshot(
                Request::get("/api/public/economics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.headers().get("cache-control").unwrap(),
            "public, max-age=300"
        );
    }

    #[tokio::test]
    async fn economics_with_orders() {
        let state = seeded_state();

        // Create an order first
        {
            let conn = state.db.lock().await;
            let restaurants: Vec<(String, String)> = {
                let mut stmt = conn
                    .prepare("SELECT r.id, r.zone_id FROM restaurants r LIMIT 1")
                    .unwrap();
                stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                    .unwrap()
                    .filter_map(|r| r.ok())
                    .collect()
            };
            let (rid, zid) = &restaurants[0];
            let oid = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO orders (id, restaurant_id, customer_address, zone_id, status,
                 food_total, delivery_fee, tip, federal_fee, local_ops_fee, processing_fee,
                 created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                rusqlite::params![
                    oid, rid, "123 Test St", zid, "Created",
                    "25.00", "5.00", "3.00", "1.00", "2.50", "0.97",
                    now, now,
                ],
            )
            .unwrap();
        }

        let app = app(state);
        let resp = app
            .oneshot(
                Request::get("/api/public/economics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(data["total_orders"], 1);
        assert_eq!(data["total_food_revenue"], "25.00");
        assert_eq!(data["total_federal_fees"], "1.00");
    }
}
