use axum::Json;
use axum::extract::State;
use serde::Serialize;
use std::collections::HashMap;
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct RevenueBreakdown {
    pub total_food_revenue: String,
    pub total_delivery_fees: String,
    pub total_federal_fees: String,
    pub total_local_ops_fees: String,
    pub total_processing_fees: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CourierUtilization {
    pub available: i64,
    pub total: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminMetrics {
    pub order_count: i64,
    pub orders_by_status: HashMap<String, i64>,
    pub on_time_delivery_rate: f64,
    pub avg_eta_error_minutes: f64,
    pub revenue_breakdown: RevenueBreakdown,
    pub courier_utilization: CourierUtilization,
    pub orders_by_zone: HashMap<String, i64>,
}

#[utoipa::path(get, path = "/admin/metrics", tag = "metrics")]
pub async fn get(State(state): State<AppState>) -> Json<AdminMetrics> {
    let conn = state.db.lock().await;

    let order_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM orders", [], |r| r.get(0))
        .unwrap_or(0);

    // Orders by status
    let mut orders_by_status = HashMap::new();
    {
        let mut stmt = conn
            .prepare("SELECT status, COUNT(*) FROM orders GROUP BY status")
            .unwrap();
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
            .unwrap();
        for row in rows.flatten() {
            orders_by_status.insert(row.0, row.1);
        }
    }

    // On-time delivery rate: orders where (julianday(actual_delivery_at) - julianday(created_at)) * 1440 < estimated_eta
    let (on_time, total_delivered) = conn
        .query_row(
            "SELECT
                SUM(CASE WHEN (julianday(actual_delivery_at) - julianday(created_at)) * 1440 < estimated_eta THEN 1 ELSE 0 END),
                COUNT(*)
             FROM orders
             WHERE actual_delivery_at IS NOT NULL AND estimated_eta IS NOT NULL",
            [],
            |r| Ok((r.get::<_, Option<i64>>(0)?, r.get::<_, i64>(1)?)),
        )
        .unwrap_or((None, 0));

    let on_time_delivery_rate = if total_delivered > 0 {
        on_time.unwrap_or(0) as f64 / total_delivered as f64 * 100.0
    } else {
        0.0
    };

    // Avg ETA error in minutes: abs((julianday(actual_delivery_at) - julianday(created_at)) * 1440 - estimated_eta)
    let avg_eta_error_minutes: f64 = conn
        .query_row(
            "SELECT COALESCE(AVG(ABS((julianday(actual_delivery_at) - julianday(created_at)) * 1440 - estimated_eta)), 0)
             FROM orders
             WHERE actual_delivery_at IS NOT NULL AND estimated_eta IS NOT NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    // Revenue breakdown
    let revenue_breakdown = conn
        .query_row(
            "SELECT
                COALESCE(SUM(CAST(food_total AS REAL)), 0),
                COALESCE(SUM(CAST(delivery_fee AS REAL)), 0),
                COALESCE(SUM(CAST(federal_fee AS REAL)), 0),
                COALESCE(SUM(CAST(local_ops_fee AS REAL)), 0),
                COALESCE(SUM(CAST(processing_fee AS REAL)), 0)
             FROM orders",
            [],
            |r| {
                Ok(RevenueBreakdown {
                    total_food_revenue: format!("{:.2}", r.get::<_, f64>(0)?),
                    total_delivery_fees: format!("{:.2}", r.get::<_, f64>(1)?),
                    total_federal_fees: format!("{:.2}", r.get::<_, f64>(2)?),
                    total_local_ops_fees: format!("{:.2}", r.get::<_, f64>(3)?),
                    total_processing_fees: format!("{:.2}", r.get::<_, f64>(4)?),
                })
            },
        )
        .unwrap_or(RevenueBreakdown {
            total_food_revenue: "0.00".into(),
            total_delivery_fees: "0.00".into(),
            total_federal_fees: "0.00".into(),
            total_local_ops_fees: "0.00".into(),
            total_processing_fees: "0.00".into(),
        });

    // Courier utilization
    let courier_utilization = conn
        .query_row(
            "SELECT
                SUM(CASE WHEN available = 1 THEN 1 ELSE 0 END),
                COUNT(*)
             FROM couriers",
            [],
            |r| {
                Ok(CourierUtilization {
                    available: r.get::<_, Option<i64>>(0)?.unwrap_or(0),
                    total: r.get(1)?,
                })
            },
        )
        .unwrap_or(CourierUtilization {
            available: 0,
            total: 0,
        });

    // Orders by zone (join to get zone name)
    let mut orders_by_zone = HashMap::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT z.name, COUNT(o.id)
                 FROM orders o
                 JOIN zones z ON o.zone_id = z.id
                 GROUP BY z.name",
            )
            .unwrap();
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
            .unwrap();
        for row in rows.flatten() {
            orders_by_zone.insert(row.0, row.1);
        }
    }

    Json(AdminMetrics {
        order_count,
        orders_by_status,
        on_time_delivery_rate,
        avg_eta_error_minutes,
        revenue_breakdown,
        courier_utilization,
        orders_by_zone,
    })
}

#[cfg(test)]
mod tests {
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
    async fn metrics_empty_db() {
        let app = app(test_state());
        let resp = app
            .oneshot(
                Request::get("/api/admin/metrics")
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
        assert_eq!(data["order_count"], 0);
        assert_eq!(data["on_time_delivery_rate"], 0.0);
        assert_eq!(data["courier_utilization"]["total"], 0);
    }

    #[tokio::test]
    async fn metrics_with_order() {
        let state = seeded_state();

        {
            let conn = state.db.lock().await;
            let mut stmt = conn
                .prepare("SELECT r.id, r.zone_id FROM restaurants r LIMIT 1")
                .unwrap();
            let (rid, zid): (String, String) =
                stmt.query_row([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
            let oid = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO orders (id, restaurant_id, customer_address, zone_id, status,
                 food_total, delivery_fee, tip, federal_fee, local_ops_fee, processing_fee,
                 created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                rusqlite::params![
                    oid,
                    rid,
                    "123 Test St",
                    zid,
                    "Created",
                    "25.00",
                    "5.00",
                    "3.00",
                    "1.00",
                    "2.50",
                    "0.97",
                    now,
                    now,
                ],
            )
            .unwrap();
        }

        let app = app(state);
        let resp = app
            .oneshot(
                Request::get("/api/admin/metrics")
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
        assert_eq!(data["order_count"], 1);
        assert!(data["orders_by_status"]["Created"].as_i64().unwrap() >= 1);
        assert_eq!(data["revenue_breakdown"]["total_food_revenue"], "25.00");
    }
}
