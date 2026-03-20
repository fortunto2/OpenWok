use openwok_core::order::Order;
use openwok_core::types::{Courier, CourierId, OrderId, Restaurant, RestaurantId};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

#[derive(Debug, Default)]
pub struct AppData {
    pub restaurants: HashMap<RestaurantId, Restaurant>,
    pub orders: HashMap<OrderId, Order>,
    pub couriers: HashMap<CourierId, Courier>,
}

#[derive(Clone, Debug, Serialize)]
pub struct OrderEvent {
    pub order_id: String,
    pub status: String,
}

#[derive(Clone)]
pub struct AppState {
    pub data: Arc<RwLock<AppData>>,
    pub order_events: broadcast::Sender<OrderEvent>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            data: Arc::new(RwLock::new(AppData::default())),
            order_events: tx,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
