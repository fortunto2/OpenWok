use serde::Deserialize;

/// Stripe API error response.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct StripeApiError {
    pub r#type: String,
    pub message: String,
    pub code: Option<String>,
}

/// All errors that can occur when talking to Stripe.
#[derive(Debug, thiserror::Error)]
pub enum StripeError {
    #[error("stripe api error: {0}")]
    Api(String),

    #[error("http error: {0}")]
    Http(String),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid webhook signature")]
    InvalidSignature,

    #[error("webhook timestamp too old")]
    TimestampTooOld,
}
