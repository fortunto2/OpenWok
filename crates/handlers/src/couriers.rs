use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use openwok_core::repo::{CreateCourierRequest, RepoError, Repository};
use openwok_core::types::{Courier, CourierId, OrderId, ZoneId};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateCourier {
    pub name: String,
    pub zone_id: ZoneId,
}

#[derive(Deserialize, ToSchema)]
pub struct SetAvailable {
    pub available: bool,
}

#[utoipa::path(get, path = "/couriers", tag = "couriers")]
pub async fn list<R: Repository>(State(repo): State<Arc<R>>) -> Json<Vec<Courier>> {
    Json(repo.list_couriers().await.unwrap_or_default())
}

#[utoipa::path(post, path = "/couriers", tag = "couriers")]
pub async fn create<R: Repository>(
    State(repo): State<Arc<R>>,
    Json(body): Json<CreateCourier>,
) -> (StatusCode, Json<Courier>) {
    let req = CreateCourierRequest {
        name: body.name,
        zone_id: body.zone_id,
    };
    let courier = repo.create_courier(req).await.unwrap();
    (StatusCode::CREATED, Json(courier))
}

#[utoipa::path(patch, path = "/couriers/{id}/available", tag = "couriers")]
pub async fn toggle_available<R: Repository>(
    State(repo): State<Arc<R>>,
    Path(id): Path<CourierId>,
    Json(body): Json<SetAvailable>,
) -> Result<Json<Courier>, StatusCode> {
    repo.toggle_courier_available(id, body.available)
        .await
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

#[utoipa::path(post, path = "/orders/{order_id}/assign", tag = "orders")]
pub async fn assign_to_order<R: Repository>(
    State(repo): State<Arc<R>>,
    Path(order_id): Path<OrderId>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = repo.assign_courier(order_id).await.map_err(|e| {
        let status = match &e {
            RepoError::NotFound => StatusCode::NOT_FOUND,
            RepoError::Conflict(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, e.to_string())
    })?;

    Ok(Json(serde_json::json!({
        "order_id": result.order_id,
        "courier_id": result.courier_id,
    })))
}
