use dioxus::prelude::*;
use openwok_core::types::Restaurant;

#[server]
pub async fn get_restaurants() -> ServerFnResult<Vec<Restaurant>> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;

    use openwok_api::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let restaurants = repo
        .list_restaurants()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(restaurants)
}

#[server]
pub async fn get_restaurant(id: String) -> ServerFnResult<Restaurant> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;
    use openwok_core::types::RestaurantId;

    use openwok_api::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let rid = RestaurantId::from_uuid(
        uuid::Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?,
    );
    let restaurant = repo
        .get_restaurant(rid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(restaurant)
}
