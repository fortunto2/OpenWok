use dioxus::prelude::*;
use openwok_core::order::{Order, OrderStatus};
use openwok_core::types::{MenuItem, MenuItemId, Restaurant, RestaurantId, ZoneId};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct CreateRestaurantInput {
    pub name: String,
    pub zone_id: String,
    pub description: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default)]
pub struct UpdateRestaurantInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default)]
#[allow(dead_code)]
pub struct UpdateMenuItemInput {
    pub name: Option<String>,
    pub price: Option<String>,
}

#[server]
pub async fn my_restaurants(token: String) -> ServerFnResult<Vec<Restaurant>> {
    let (repo, user) = authenticated_repo(&token).await?;
    repo.list_restaurants_by_owner(user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn my_orders(token: String) -> ServerFnResult<Vec<Order>> {
    let (repo, user) = authenticated_repo(&token).await?;
    let restaurants = repo
        .list_restaurants_by_owner(user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut orders = Vec::new();
    for restaurant in &restaurants {
        let mut restaurant_orders = repo
            .list_restaurant_orders(restaurant.id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        orders.append(&mut restaurant_orders);
    }
    orders.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(orders)
}

#[server]
pub async fn create_restaurant(
    token: String,
    input: CreateRestaurantInput,
) -> ServerFnResult<Restaurant> {
    use openwok_core::repo::CreateRestaurantRequest;
    use openwok_core::types::UserRole;

    let (repo, user) = authenticated_repo(&token).await?;
    let restaurant = repo
        .create_restaurant(CreateRestaurantRequest {
            name: input.name,
            zone_id: parse_zone_id(&input.zone_id)?,
            menu: Vec::new(),
            owner_id: Some(user.id),
            description: input.description,
            address: input.address,
            phone: input.phone,
        })
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if user.role == UserRole::Customer {
        let _ = repo
            .update_user_role(user.id, UserRole::RestaurantOwner)
            .await;
    }

    Ok(restaurant)
}

#[server]
pub async fn update_restaurant(
    token: String,
    id: String,
    input: UpdateRestaurantInput,
) -> ServerFnResult<Restaurant> {
    use openwok_core::types::UpdateRestaurantRequest;

    let (repo, user) = authenticated_repo(&token).await?;
    let restaurant_id = parse_restaurant_id(&id)?;
    verify_restaurant_ownership(repo.as_ref(), user.id, restaurant_id).await?;

    repo.update_restaurant(
        restaurant_id,
        UpdateRestaurantRequest {
            name: input.name,
            description: input.description,
            address: input.address,
            phone: input.phone,
        },
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn toggle_restaurant_active(
    token: String,
    id: String,
    active: bool,
) -> ServerFnResult<Restaurant> {
    let (repo, user) = authenticated_repo(&token).await?;
    let restaurant_id = parse_restaurant_id(&id)?;
    verify_restaurant_ownership(repo.as_ref(), user.id, restaurant_id).await?;

    repo.toggle_restaurant_active(restaurant_id, active)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn add_menu_item(
    token: String,
    restaurant_id: String,
    name: String,
    price: String,
) -> ServerFnResult<MenuItem> {
    use openwok_core::money::Money;
    use openwok_core::repo::CreateMenuItemRequest;

    let (repo, user) = authenticated_repo(&token).await?;
    let restaurant_id = parse_restaurant_id(&restaurant_id)?;
    verify_restaurant_ownership(repo.as_ref(), user.id, restaurant_id).await?;

    repo.add_menu_item(
        restaurant_id,
        CreateMenuItemRequest {
            name,
            price: Money::from(price.as_str()),
        },
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn update_menu_item(
    token: String,
    item_id: String,
    input: UpdateMenuItemInput,
) -> ServerFnResult<MenuItem> {
    use openwok_core::money::Money;
    use openwok_core::types::UpdateMenuItemRequest;

    let (repo, user) = authenticated_repo(&token).await?;
    let item_id = parse_menu_item_id(&item_id)?;
    let current = repo
        .get_menu_item(item_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    verify_restaurant_ownership(repo.as_ref(), user.id, current.restaurant_id).await?;

    repo.update_menu_item(
        item_id,
        UpdateMenuItemRequest {
            name: input.name,
            price: input.price.map(|price| Money::from(price.as_str())),
        },
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn delete_menu_item(token: String, item_id: String) -> ServerFnResult<()> {
    let (repo, user) = authenticated_repo(&token).await?;
    let item_id = parse_menu_item_id(&item_id)?;
    let item = repo
        .get_menu_item(item_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    verify_restaurant_ownership(repo.as_ref(), user.id, item.restaurant_id).await?;
    repo.delete_menu_item(item_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server]
pub async fn update_owned_order_status(
    token: String,
    order_id: String,
    status: String,
) -> ServerFnResult<Order> {
    let (repo, user) = authenticated_repo(&token).await?;
    let order_id = parse_order_id(&order_id)?;
    let order = repo
        .get_order(order_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    verify_restaurant_ownership(repo.as_ref(), user.id, order.restaurant_id).await?;
    let status = parse_order_status(&status)?;
    repo.update_order_status(order_id, status)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[cfg(feature = "server")]
async fn authenticated_repo(
    token: &str,
) -> ServerFnResult<(
    std::sync::Arc<openwok_api::SqliteRepo>,
    openwok_core::types::User,
)> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;

    use openwok_api::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let user = crate::server_fns::auth::verify_token_and_get_user(token, repo.as_ref()).await?;
    Ok((repo, user))
}

#[cfg(feature = "server")]
async fn verify_restaurant_ownership<R: openwok_core::repo::Repository>(
    repo: &R,
    user_id: openwok_core::types::UserId,
    restaurant_id: RestaurantId,
) -> ServerFnResult<()> {
    let restaurant = repo
        .get_restaurant(restaurant_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    match restaurant.owner_id {
        Some(owner_id) if owner_id == user_id => Ok(()),
        _ => Err(ServerFnError::new("not the owner")),
    }
}

#[allow(dead_code)]
fn parse_restaurant_id(value: &str) -> ServerFnResult<RestaurantId> {
    uuid::Uuid::parse_str(value)
        .map(RestaurantId::from_uuid)
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[allow(dead_code)]
fn parse_zone_id(value: &str) -> ServerFnResult<ZoneId> {
    uuid::Uuid::parse_str(value)
        .map(ZoneId::from_uuid)
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[allow(dead_code)]
fn parse_menu_item_id(value: &str) -> ServerFnResult<MenuItemId> {
    uuid::Uuid::parse_str(value)
        .map(MenuItemId::from_uuid)
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[allow(dead_code)]
fn parse_order_id(value: &str) -> ServerFnResult<openwok_core::types::OrderId> {
    uuid::Uuid::parse_str(value)
        .map(openwok_core::types::OrderId::from_uuid)
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[allow(dead_code)]
fn parse_order_status(value: &str) -> ServerFnResult<OrderStatus> {
    match value {
        "Confirmed" => Ok(OrderStatus::Confirmed),
        "Preparing" => Ok(OrderStatus::Preparing),
        "ReadyForPickup" => Ok(OrderStatus::ReadyForPickup),
        "InDelivery" => Ok(OrderStatus::InDelivery),
        "Delivered" => Ok(OrderStatus::Delivered),
        "Cancelled" => Ok(OrderStatus::Cancelled),
        _ => Err(ServerFnError::new(format!("Invalid status: {value}"))),
    }
}
