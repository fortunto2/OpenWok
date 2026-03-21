use dioxus::prelude::*;
use openwok_core::order::{Order, OrderStatus};
use openwok_core::types::{Courier, Dispute, DisputeId, DisputeStatus, OrderId, User, UserId};

#[server]
pub async fn list_orders() -> ServerFnResult<Vec<Order>> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;

    use openwok_api::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    repo.list_orders()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn assign_courier(order_id: String) -> ServerFnResult<()> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;

    use openwok_api::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let order_id = parse_order_id(&order_id)?;
    repo.assign_courier(order_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server]
pub async fn transition_order_status(id: String, status: String) -> ServerFnResult<Order> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;

    use openwok_api::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let order_id = parse_order_id(&id)?;
    let new_status = parse_order_status(&status)?;
    repo.update_order_status(order_id, new_status)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_couriers() -> ServerFnResult<Vec<Courier>> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;

    use openwok_api::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    repo.list_couriers()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_users() -> ServerFnResult<Vec<User>> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;

    use openwok_api::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    repo.list_users()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn set_user_blocked(user_id: String, blocked: bool) -> ServerFnResult<User> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;

    use openwok_api::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let user_id = parse_user_id(&user_id)?;
    repo.set_user_blocked(user_id, blocked)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_disputes() -> ServerFnResult<Vec<Dispute>> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;

    use openwok_api::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    repo.list_disputes()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn resolve_dispute(
    dispute_id: String,
    status: String,
    resolution: Option<String>,
) -> ServerFnResult<Dispute> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;

    use openwok_api::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let dispute_id = parse_dispute_id(&dispute_id)?;
    let status = parse_dispute_status(&status)?;
    repo.resolve_dispute(dispute_id, status, resolution)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[allow(dead_code)]
fn parse_order_id(value: &str) -> ServerFnResult<OrderId> {
    uuid::Uuid::parse_str(value)
        .map(OrderId::from_uuid)
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[allow(dead_code)]
fn parse_user_id(value: &str) -> ServerFnResult<UserId> {
    uuid::Uuid::parse_str(value)
        .map(UserId::from_uuid)
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[allow(dead_code)]
fn parse_dispute_id(value: &str) -> ServerFnResult<DisputeId> {
    uuid::Uuid::parse_str(value)
        .map(DisputeId::from_uuid)
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[allow(dead_code)]
fn parse_order_status(status: &str) -> ServerFnResult<OrderStatus> {
    match status {
        "Confirmed" => Ok(OrderStatus::Confirmed),
        "Preparing" => Ok(OrderStatus::Preparing),
        "ReadyForPickup" => Ok(OrderStatus::ReadyForPickup),
        "InDelivery" => Ok(OrderStatus::InDelivery),
        "Delivered" => Ok(OrderStatus::Delivered),
        "Cancelled" => Ok(OrderStatus::Cancelled),
        _ => Err(ServerFnError::new(format!("Invalid status: {status}"))),
    }
}

#[allow(dead_code)]
fn parse_dispute_status(status: &str) -> ServerFnResult<DisputeStatus> {
    match status {
        "Open" => Ok(DisputeStatus::Open),
        "Resolved" => Ok(DisputeStatus::Resolved),
        "Dismissed" => Ok(DisputeStatus::Dismissed),
        _ => Err(ServerFnError::new(format!(
            "Invalid dispute status: {status}"
        ))),
    }
}
