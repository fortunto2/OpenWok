use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use openwok_core::repo::{AdminMetrics, Repository};

#[utoipa::path(get, path = "/admin/metrics", tag = "metrics")]
pub async fn get<R: Repository>(State(repo): State<Arc<R>>) -> Json<AdminMetrics> {
    Json(repo.get_metrics().await.unwrap_or(AdminMetrics {
        order_count: 0,
        orders_by_status: Default::default(),
        on_time_delivery_rate: 0.0,
        avg_eta_error_minutes: 0.0,
        revenue_breakdown: openwok_core::repo::RevenueBreakdown {
            total_food_revenue: "0.00".into(),
            total_delivery_fees: "0.00".into(),
            total_federal_fees: "0.00".into(),
            total_local_ops_fees: "0.00".into(),
            total_processing_fees: "0.00".into(),
        },
        courier_utilization: openwok_core::repo::CourierUtilization {
            available: 0,
            total: 0,
        },
        orders_by_zone: Default::default(),
    }))
}
