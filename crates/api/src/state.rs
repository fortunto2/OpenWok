use openwok_core::order::Order;
use openwok_core::types::{Courier, CourierId, OrderId, Restaurant, RestaurantId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct AppState {
    pub restaurants: HashMap<RestaurantId, Restaurant>,
    pub orders: HashMap<OrderId, Order>,
    pub couriers: HashMap<CourierId, Courier>,
}

pub type SharedState = Arc<RwLock<AppState>>;
