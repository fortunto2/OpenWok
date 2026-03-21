use axum::Json;

/// Platform config — fees, version, defaults.
/// For MVP: hardcoded defaults. Future: read from Node config in DB.
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct PlatformConfig {
    pub delivery_fee: String,
    pub local_ops_fee: String,
    pub federal_fee: String,
    pub default_tip: String,
    pub api_version: u32,
}

#[utoipa::path(get, path = "/config", tag = "config")]
pub async fn get() -> Json<PlatformConfig> {
    Json(PlatformConfig {
        delivery_fee: "5.00".into(),
        local_ops_fee: "2.50".into(),
        federal_fee: "1.00".into(),
        default_tip: "3.00".into(),
        api_version: 1,
    })
}
