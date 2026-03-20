use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use openwok_core::money::Money;
use openwok_core::order::Order;
use openwok_core::repo::{CreateMenuItemRequest, CreateRestaurantRequest, RepoError, Repository};
use openwok_core::types::{
    MenuItemId, Restaurant, RestaurantId, UpdateMenuItemRequest, UpdateRestaurantRequest, UserRole,
    ZoneId,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::admin::get_active_user;
use crate::auth::AuthUser;

// --- Request DTOs ---

#[derive(Deserialize, ToSchema)]
pub struct CreateRestaurant {
    pub name: String,
    pub zone_id: ZoneId,
    pub menu: Vec<CreateMenuItem>,
    pub description: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateMenuItem {
    pub name: String,
    pub price: Money,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateRestaurant {
    pub name: Option<String>,
    pub description: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct ToggleActive {
    pub active: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateMenuItem {
    pub name: Option<String>,
    pub price: Option<Money>,
}

// --- Helpers ---

pub(crate) fn repo_error_to_status(e: &RepoError) -> StatusCode {
    match e {
        RepoError::NotFound => StatusCode::NOT_FOUND,
        RepoError::InvalidTransition(_) => StatusCode::BAD_REQUEST,
        RepoError::Conflict(_) => StatusCode::BAD_REQUEST,
        RepoError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Check if the authenticated user is active (non-blocked) and owns the given restaurant.
async fn verify_ownership<R: Repository>(
    repo: &R,
    auth: &AuthUser,
    restaurant_id: RestaurantId,
) -> Result<(), (StatusCode, String)> {
    let user = get_active_user(repo, auth).await?;

    let restaurant = repo
        .get_restaurant(restaurant_id)
        .await
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))?;

    match restaurant.owner_id {
        Some(owner_id) if owner_id == user.id => Ok(()),
        _ => Err((StatusCode::FORBIDDEN, "not the owner".into())),
    }
}

// --- Public endpoints ---

#[utoipa::path(get, path = "/restaurants", tag = "restaurants")]
pub async fn list<R: Repository>(State(repo): State<Arc<R>>) -> Json<Vec<Restaurant>> {
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

// --- Authenticated endpoints ---

/// POST /api/restaurants — create restaurant (auth required, auto-promote to RestaurantOwner)
#[utoipa::path(post, path = "/restaurants", tag = "restaurants")]
pub async fn create<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
    Json(body): Json<CreateRestaurant>,
) -> Result<(StatusCode, Json<Restaurant>), (StatusCode, String)> {
    let user = get_active_user(repo.as_ref(), &auth).await?;

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
        owner_id: Some(user.id),
        description: body.description,
        address: body.address,
        phone: body.phone,
    };

    let restaurant = repo
        .create_restaurant(req)
        .await
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))?;

    // Auto-promote to RestaurantOwner if currently Customer
    if user.role == UserRole::Customer {
        let _ = repo
            .update_user_role(user.id, UserRole::RestaurantOwner)
            .await;
    }

    Ok((StatusCode::CREATED, Json(restaurant)))
}

/// PATCH /api/restaurants/:id — update restaurant info (owner only)
#[utoipa::path(patch, path = "/restaurants/{id}", tag = "restaurants")]
pub async fn update<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
    Path(id): Path<RestaurantId>,
    Json(body): Json<UpdateRestaurant>,
) -> Result<Json<Restaurant>, (StatusCode, String)> {
    verify_ownership(&*repo, &auth, id).await?;

    let req = UpdateRestaurantRequest {
        name: body.name,
        description: body.description,
        address: body.address,
        phone: body.phone,
    };

    repo.update_restaurant(id, req)
        .await
        .map(Json)
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))
}

/// PATCH /api/restaurants/:id/active — toggle active status (owner only)
#[utoipa::path(patch, path = "/restaurants/{id}/active", tag = "restaurants")]
pub async fn toggle_active<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
    Path(id): Path<RestaurantId>,
    Json(body): Json<ToggleActive>,
) -> Result<Json<Restaurant>, (StatusCode, String)> {
    verify_ownership(&*repo, &auth, id).await?;

    repo.toggle_restaurant_active(id, body.active)
        .await
        .map(Json)
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))
}

/// POST /api/restaurants/:id/menu — add menu item (owner only)
#[utoipa::path(post, path = "/restaurants/{id}/menu", tag = "restaurants")]
pub async fn add_menu_item<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
    Path(id): Path<RestaurantId>,
    Json(body): Json<CreateMenuItem>,
) -> Result<(StatusCode, Json<openwok_core::types::MenuItem>), (StatusCode, String)> {
    verify_ownership(&*repo, &auth, id).await?;

    let req = CreateMenuItemRequest {
        name: body.name,
        price: body.price,
    };

    repo.add_menu_item(id, req)
        .await
        .map(|item| (StatusCode::CREATED, Json(item)))
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))
}

/// PATCH /api/menu-items/:id — update menu item (owner only)
#[utoipa::path(patch, path = "/menu-items/{id}", tag = "restaurants")]
pub async fn update_menu_item<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
    Path(id): Path<MenuItemId>,
    Json(body): Json<UpdateMenuItem>,
) -> Result<Json<openwok_core::types::MenuItem>, (StatusCode, String)> {
    // Look up menu item to find its restaurant, then verify ownership BEFORE modifying
    let current = repo
        .get_menu_item(id)
        .await
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))?;

    verify_ownership(&*repo, &auth, current.restaurant_id).await?;

    let req = UpdateMenuItemRequest {
        name: body.name,
        price: body.price,
    };

    let item = repo
        .update_menu_item(id, req)
        .await
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))?;

    Ok(Json(item))
}

/// DELETE /api/menu-items/:id — delete menu item (owner only)
#[utoipa::path(delete, path = "/menu-items/{id}", tag = "restaurants")]
pub async fn delete_menu_item<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
    Path(id): Path<MenuItemId>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Look up menu item to verify ownership BEFORE deleting
    let item = repo
        .get_menu_item(id)
        .await
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))?;

    verify_ownership(&*repo, &auth, item.restaurant_id).await?;

    repo.delete_menu_item(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))
}

/// GET /api/my/restaurants — list restaurants owned by current user
#[utoipa::path(get, path = "/my/restaurants", tag = "restaurants")]
pub async fn my_restaurants<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
) -> Result<Json<Vec<Restaurant>>, (StatusCode, String)> {
    let user = get_active_user(repo.as_ref(), &auth).await?;

    repo.list_restaurants_by_owner(user.id)
        .await
        .map(Json)
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))
}

/// GET /api/my/orders — list orders across all restaurants owned by current user
#[utoipa::path(get, path = "/my/orders", tag = "restaurants")]
pub async fn my_orders<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
) -> Result<Json<Vec<Order>>, (StatusCode, String)> {
    let user = get_active_user(repo.as_ref(), &auth).await?;

    let restaurants = repo
        .list_restaurants_by_owner(user.id)
        .await
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))?;

    let mut all_orders = Vec::new();
    for restaurant in &restaurants {
        let orders = repo
            .list_restaurant_orders(restaurant.id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        all_orders.extend(orders);
    }

    // Sort all orders by created_at DESC across restaurants
    all_orders.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(Json(all_orders))
}
