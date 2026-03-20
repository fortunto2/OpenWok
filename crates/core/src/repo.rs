use std::collections::HashMap;

use async_trait::async_trait;

use crate::money::Money;
use crate::order::{Order, OrderStatus};
use crate::types::{Courier, CourierId, MenuItemId, OrderId, Restaurant, RestaurantId, ZoneId};

/// Repository errors — maps to HTTP statuses in handlers.
#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("not found")]
    NotFound,
    #[error("invalid transition: {0}")]
    InvalidTransition(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal: {0}")]
    Internal(String),
}

/// Request types for creating entities through the Repository.
#[derive(Debug, Clone)]
pub struct CreateRestaurantRequest {
    pub name: String,
    pub zone_id: ZoneId,
    pub menu: Vec<CreateMenuItemRequest>,
}

#[derive(Debug, Clone)]
pub struct CreateMenuItemRequest {
    pub name: String,
    pub price: Money,
}

#[derive(Debug, Clone)]
pub struct CreateOrderRequest {
    pub restaurant_id: RestaurantId,
    pub items: Vec<CreateOrderItemRequest>,
    pub customer_address: String,
    pub zone_id: ZoneId,
    pub delivery_fee: Money,
    pub tip: Money,
    pub local_ops_fee: Money,
}

#[derive(Debug, Clone)]
pub struct CreateOrderItemRequest {
    pub menu_item_id: MenuItemId,
    pub name: String,
    pub quantity: u32,
    pub unit_price: Money,
}

#[derive(Debug, Clone)]
pub struct CreateCourierRequest {
    pub name: String,
    pub zone_id: ZoneId,
}

/// Public economics aggregation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct PublicEconomics {
    pub total_orders: i64,
    pub total_food_revenue: String,
    pub total_delivery_fees: String,
    pub total_federal_fees: String,
    pub total_local_ops_fees: String,
    pub total_processing_fees: String,
    pub avg_order_value: String,
}

/// Admin metrics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AdminMetrics {
    pub order_count: i64,
    pub orders_by_status: HashMap<String, i64>,
    pub on_time_delivery_rate: f64,
    pub avg_eta_error_minutes: f64,
    pub revenue_breakdown: RevenueBreakdown,
    pub courier_utilization: CourierUtilization,
    pub orders_by_zone: HashMap<String, i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct RevenueBreakdown {
    pub total_food_revenue: String,
    pub total_delivery_fees: String,
    pub total_federal_fees: String,
    pub total_local_ops_fees: String,
    pub total_processing_fees: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CourierUtilization {
    pub available: i64,
    pub total: i64,
}

/// Courier assignment result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssignCourierResult {
    pub order_id: String,
    pub courier_id: String,
}

/// The data access abstraction. Implemented by SqliteRepo (local) and D1Repo (worker).
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    // Restaurants
    async fn list_restaurants(&self) -> Result<Vec<Restaurant>, RepoError>;
    async fn get_restaurant(&self, id: RestaurantId) -> Result<Restaurant, RepoError>;
    async fn create_restaurant(&self, req: CreateRestaurantRequest) -> Result<Restaurant, RepoError>;

    // Orders
    async fn list_orders(&self) -> Result<Vec<Order>, RepoError>;
    async fn get_order(&self, id: OrderId) -> Result<Order, RepoError>;
    async fn create_order(&self, req: CreateOrderRequest) -> Result<Order, RepoError>;
    async fn update_order_status(&self, id: OrderId, status: OrderStatus) -> Result<Order, RepoError>;
    async fn assign_courier(&self, order_id: OrderId) -> Result<AssignCourierResult, RepoError>;

    // Couriers
    async fn list_couriers(&self) -> Result<Vec<Courier>, RepoError>;
    async fn create_courier(&self, req: CreateCourierRequest) -> Result<Courier, RepoError>;
    async fn toggle_courier_available(&self, id: CourierId, available: bool) -> Result<Courier, RepoError>;

    // Economics & Metrics
    async fn get_economics(&self) -> Result<PublicEconomics, RepoError>;
    async fn get_metrics(&self) -> Result<AdminMetrics, RepoError>;
}
