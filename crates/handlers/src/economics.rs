use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use openwok_core::repo::{PublicEconomics, Repository};

#[utoipa::path(get, path = "/public/economics", tag = "economics")]
pub async fn get<R: Repository>(State(repo): State<Arc<R>>) -> (HeaderMap, Json<PublicEconomics>) {
    let economics = repo.get_economics().await.unwrap_or(PublicEconomics {
        total_orders: 0,
        total_food_revenue: "0.00".into(),
        total_delivery_fees: "0.00".into(),
        total_federal_fees: "0.00".into(),
        total_local_ops_fees: "0.00".into(),
        total_processing_fees: "0.00".into(),
        avg_order_value: "0.00".into(),
    });

    let mut headers = HeaderMap::new();
    headers.insert("Cache-Control", "public, max-age=300".parse().unwrap());

    (headers, Json(economics))
}
