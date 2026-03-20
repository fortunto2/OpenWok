use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use openwok_core::money::Money;
use openwok_core::repo::{CreateMenuItemRequest, CreateRestaurantRequest, RepoError, Repository};
use openwok_core::types::{Restaurant, RestaurantId, ZoneId};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateRestaurant {
    pub name: String,
    pub zone_id: ZoneId,
    pub menu: Vec<CreateMenuItem>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateMenuItem {
    pub name: String,
    pub price: Money,
}

#[utoipa::path(get, path = "/restaurants", tag = "restaurants")]
pub async fn list<R: Repository>(State(repo): State<Arc<R>>) -> Json<Vec<Restaurant>> {
    // list_restaurants shouldn't fail in practice; unwrap_or empty
    Json(repo.list_restaurants().await.unwrap_or_default())
}

#[utoipa::path(get, path = "/restaurants/{id}", tag = "restaurants")]
pub async fn get<R: Repository>(
    State(repo): State<Arc<R>>,
    Path(id): Path<RestaurantId>,
) -> Result<Json<Restaurant>, StatusCode> {
    repo.get_restaurant(id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

#[utoipa::path(post, path = "/restaurants", tag = "restaurants")]
pub async fn create<R: Repository>(
    State(repo): State<Arc<R>>,
    Json(body): Json<CreateRestaurant>,
) -> (StatusCode, Json<Restaurant>) {
    let req = CreateRestaurantRequest {
        name: body.name,
        zone_id: body.zone_id,
        menu: body
            .menu
            .into_iter()
            .map(|m| CreateMenuItemRequest {
                name: m.name,
                price: m.price,
            })
            .collect(),
    };
    let restaurant = repo.create_restaurant(req).await.unwrap();
    (StatusCode::CREATED, Json(restaurant))
}

pub(crate) fn repo_error_to_status(e: &RepoError) -> StatusCode {
    match e {
        RepoError::NotFound => StatusCode::NOT_FOUND,
        RepoError::InvalidTransition(_) => StatusCode::BAD_REQUEST,
        RepoError::Conflict(_) => StatusCode::BAD_REQUEST,
        RepoError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
