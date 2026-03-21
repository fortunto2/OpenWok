use dioxus::prelude::*;
use openwok_core::money::Money;
use openwok_core::order::Order;
use openwok_core::types::{MenuItemId, RestaurantId, ZoneId};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[allow(dead_code)]
pub struct OrderItemInput {
    pub menu_item_id: MenuItemId,
    pub name: String,
    pub quantity: u32,
    pub unit_price: Money,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[allow(dead_code)]
pub struct CreateOrderInput {
    pub restaurant_id: RestaurantId,
    pub items: Vec<OrderItemInput>,
    pub customer_address: String,
    pub zone_id: ZoneId,
    pub delivery_fee: Money,
    pub tip: Money,
    pub local_ops_fee: Money,
}

#[server]
pub async fn create_order(input: CreateOrderInput) -> ServerFnResult<Order> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;

    use crate::db::repo::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let order_req = openwok_core::repo::CreateOrderRequest {
        restaurant_id: input.restaurant_id,
        items: input
            .items
            .into_iter()
            .map(|i| openwok_core::repo::CreateOrderItemRequest {
                menu_item_id: i.menu_item_id,
                name: i.name,
                quantity: i.quantity,
                unit_price: i.unit_price,
            })
            .collect(),
        customer_address: input.customer_address,
        zone_id: input.zone_id,
        delivery_fee: input.delivery_fee,
        tip: input.tip,
        local_ops_fee: input.local_ops_fee,
    };
    let order = repo
        .create_order(order_req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(order)
}

#[server]
pub async fn get_order(id: String) -> ServerFnResult<Order> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;
    use openwok_core::types::OrderId;

    use crate::db::repo::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let oid = OrderId::from_uuid(
        uuid::Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?,
    );
    let order = repo
        .get_order(oid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(order)
}

#[server]
pub async fn transition_order(id: String, status: String) -> ServerFnResult<Order> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::order::OrderStatus;
    use openwok_core::repo::Repository;
    use openwok_core::types::OrderId;

    use crate::db::repo::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let oid = OrderId::from_uuid(
        uuid::Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?,
    );
    let new_status = match status.as_str() {
        "Confirmed" => OrderStatus::Confirmed,
        "Preparing" => OrderStatus::Preparing,
        "ReadyForPickup" => OrderStatus::ReadyForPickup,
        "InDelivery" => OrderStatus::InDelivery,
        "Delivered" => OrderStatus::Delivered,
        "Cancelled" => OrderStatus::Cancelled,
        _ => return Err(ServerFnError::new(format!("Invalid status: {status}"))),
    };
    let order = repo
        .update_order_status(oid, new_status)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(order)
}
