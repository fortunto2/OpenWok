use dioxus::prelude::*;
use openwok_core::order::Order;
use openwok_core::types::Courier;

#[server]
pub async fn register_courier(
    token: String,
    name: String,
    zone_id: String,
) -> ServerFnResult<Courier> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::{CreateCourierRequest, Repository};

    use openwok_api::SqliteRepo;
    use openwok_core::types::{UserRole, ZoneId};

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let user = crate::server_fns::auth::verify_token_and_get_user(&token, repo.as_ref()).await?;
    let zid = ZoneId::from_uuid(
        uuid::Uuid::parse_str(&zone_id).map_err(|e| ServerFnError::new(e.to_string()))?,
    );
    let req = CreateCourierRequest {
        name,
        zone_id: zid,
        user_id: Some(user.id.to_string()),
    };
    let courier = repo
        .create_courier(req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let _ = repo.update_user_role(user.id, UserRole::Courier).await;
    Ok(courier)
}

#[server]
pub async fn get_courier_me(token: String) -> ServerFnResult<Courier> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;

    use openwok_api::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let user = crate::server_fns::auth::verify_token_and_get_user(&token, repo.as_ref()).await?;
    let courier = repo
        .get_courier_by_user_id(&user.id.to_string())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(courier)
}

#[server]
pub async fn toggle_availability(courier_id: String, available: bool) -> ServerFnResult<Courier> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;
    use openwok_core::types::CourierId;

    use openwok_api::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let cid = CourierId::from_uuid(
        uuid::Uuid::parse_str(&courier_id).map_err(|e| ServerFnError::new(e.to_string()))?,
    );
    let courier = repo
        .toggle_courier_available(cid, available)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(courier)
}

#[server]
pub async fn get_my_deliveries(token: String) -> ServerFnResult<Vec<Order>> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;

    use openwok_api::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let user = crate::server_fns::auth::verify_token_and_get_user(&token, repo.as_ref()).await?;
    let courier = repo
        .get_courier_by_user_id(&user.id.to_string())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let orders = repo
        .list_courier_orders(courier.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(orders)
}

#[server]
pub async fn set_my_availability(token: String, available: bool) -> ServerFnResult<Courier> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;

    use openwok_api::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let user = crate::server_fns::auth::verify_token_and_get_user(&token, repo.as_ref()).await?;
    let courier = repo
        .get_courier_by_user_id(&user.id.to_string())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    repo.toggle_courier_available(courier.id, available)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn mark_delivery_completed(token: String, order_id: String) -> ServerFnResult<Order> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::order::OrderStatus;
    use openwok_core::repo::Repository;
    use openwok_core::types::OrderId;

    use openwok_api::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let user = crate::server_fns::auth::verify_token_and_get_user(&token, repo.as_ref()).await?;
    let courier = repo
        .get_courier_by_user_id(&user.id.to_string())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let order_id = OrderId::from_uuid(
        uuid::Uuid::parse_str(&order_id).map_err(|e| ServerFnError::new(e.to_string()))?,
    );
    let order = repo
        .get_order(order_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if order.courier_id != Some(courier.id) {
        return Err(ServerFnError::new(
            "order is not assigned to current courier",
        ));
    }

    repo.update_order_status(order_id, OrderStatus::Delivered)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
