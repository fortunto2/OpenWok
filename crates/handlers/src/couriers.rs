use std::sync::Arc;

use crate::admin::get_active_user;
use crate::auth::AuthUser;
use crate::restaurants::repo_error_to_status;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use openwok_core::dispatch::OrderEvent;
use openwok_core::order::Order;
use openwok_core::repo::{CreateCourierRequest, RepoError, Repository};
use openwok_core::types::{Courier, CourierId, OrderId, UserRole, ZoneId};
use serde::Deserialize;
use tokio::sync::broadcast;
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
    auth: AuthUser,
    State(repo): State<Arc<R>>,
    Json(body): Json<CreateCourier>,
) -> Result<(StatusCode, Json<Courier>), (StatusCode, String)> {
    let user = get_active_user(repo.as_ref(), &auth).await?;

    let req = CreateCourierRequest {
        name: body.name,
        zone_id: body.zone_id,
        user_id: Some(user.id.to_string()),
    };
    let courier = repo
        .create_courier(req)
        .await
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))?;

    // Auto-promote user role to Courier
    let _ = repo.update_user_role(user.id, UserRole::Courier).await;

    Ok((StatusCode::CREATED, Json(courier)))
}

#[utoipa::path(patch, path = "/couriers/{id}/available", tag = "couriers")]
pub async fn toggle_available<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
    Path(id): Path<CourierId>,
    Json(body): Json<SetAvailable>,
) -> Result<Json<Courier>, (StatusCode, String)> {
    get_active_user(repo.as_ref(), &auth).await?;
    repo.toggle_courier_available(id, body.available)
        .await
        .map(Json)
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))
}

#[utoipa::path(post, path = "/orders/{order_id}/assign", tag = "orders")]
pub async fn assign_to_order<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
    tx: Option<axum::Extension<broadcast::Sender<OrderEvent>>>,
    Path(order_id): Path<OrderId>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    get_active_user(repo.as_ref(), &auth).await?;
    let result = repo.assign_courier(order_id).await.map_err(|e| {
        let status = match &e {
            RepoError::NotFound => StatusCode::NOT_FOUND,
            RepoError::Conflict(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, e.to_string())
    })?;

    // Broadcast courier assignment event
    if let Some(axum::Extension(sender)) = tx {
        let _ = sender.send(OrderEvent {
            order_id: result.order_id.clone(),
            status: "CourierAssigned".into(),
        });
    }

    Ok(Json(serde_json::json!({
        "order_id": result.order_id,
        "courier_id": result.courier_id,
    })))
}

/// GET /api/couriers/me — get current user's courier profile
#[utoipa::path(get, path = "/couriers/me", tag = "couriers")]
pub async fn me<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
) -> Result<Json<Courier>, (StatusCode, String)> {
    let user = get_active_user(repo.as_ref(), &auth).await?;

    repo.get_courier_by_user_id(&user.id.to_string())
        .await
        .map(Json)
        .map_err(|_| (StatusCode::NOT_FOUND, "not registered as courier".into()))
}

/// GET /api/my/deliveries — list orders assigned to current courier
#[utoipa::path(get, path = "/my/deliveries", tag = "couriers")]
pub async fn my_deliveries<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
) -> Result<Json<Vec<Order>>, (StatusCode, String)> {
    let user = get_active_user(repo.as_ref(), &auth).await?;

    let courier = repo
        .get_courier_by_user_id(&user.id.to_string())
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "not registered as courier".into()))?;

    repo.list_courier_orders(courier.id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
