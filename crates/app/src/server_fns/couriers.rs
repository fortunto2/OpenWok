use dioxus::prelude::*;
use openwok_core::order::Order;
use openwok_core::types::Courier;

#[server]
pub async fn register_courier(name: String, zone_id: String) -> ServerFnResult<Courier> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::{CreateCourierRequest, Repository};

    use crate::db::repo::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let zid = ZoneId::from_uuid(
        uuid::Uuid::parse_str(&zone_id).map_err(|e| ServerFnError::new(e.to_string()))?,
    );
    let req = CreateCourierRequest {
        name,
        zone_id: zid,
        user_id: None,
    };
    let courier = repo
        .create_courier(req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(courier)
}

#[server]
pub async fn get_courier_me(user_id: String) -> ServerFnResult<Courier> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;

    use crate::db::repo::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let courier = repo
        .get_courier_by_user_id(&user_id)
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

    use crate::db::repo::SqliteRepo;

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
pub async fn get_my_deliveries(courier_id: String) -> ServerFnResult<Vec<Order>> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;
    use openwok_core::types::CourierId;

    use crate::db::repo::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let cid = CourierId::from_uuid(
        uuid::Uuid::parse_str(&courier_id).map_err(|e| ServerFnError::new(e.to_string()))?,
    );
    let orders = repo
        .list_courier_orders(cid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(orders)
}
