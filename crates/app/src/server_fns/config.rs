use dioxus::prelude::*;
use openwok_core::repo::{AdminMetrics, PublicEconomics};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[allow(dead_code)]
pub struct PlatformConfig {
    pub federal_fee: String,
    pub default_local_ops_fee: String,
    pub default_delivery_fee: String,
}

#[server]
pub async fn get_config() -> ServerFnResult<PlatformConfig> {
    Ok(PlatformConfig {
        federal_fee: "1.00".into(),
        default_local_ops_fee: "2.50".into(),
        default_delivery_fee: "5.00".into(),
    })
}

#[server]
pub async fn get_economics() -> ServerFnResult<PublicEconomics> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;

    use crate::db::repo::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let economics = repo
        .get_economics()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(economics)
}

#[server]
pub async fn get_admin_metrics() -> ServerFnResult<AdminMetrics> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;

    use crate::db::repo::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let metrics = repo
        .get_metrics()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(metrics)
}
