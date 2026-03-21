//! Axum middleware for JWT authentication.
//!
//! ```rust,ignore
//! use superduperai_auth::middleware::AuthLayer;
//! use superduperai_auth::{AuthClient, AuthConfig};
//!
//! let config = AuthConfig::from_env("openwok").unwrap();
//! let auth = AuthClient::new(config);
//!
//! let app = Router::new()
//!     .route("/api/orders", get(list_orders))
//!     .layer(AuthLayer::new(auth));
//! ```

use crate::AuthClient;
use crate::error::AuthError;
use crate::types::Claims;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Authenticated user extracted from JWT.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub email: Option<String>,
    pub claims: Claims,
}

/// Axum middleware that verifies JWT and injects AuthUser.
pub async fn require_auth(
    axum::extract::State(auth): axum::extract::State<Arc<AuthClient>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AuthError::MissingAuth)?;

    let claims = auth.verify_token(token)?;

    let auth_user = AuthUser {
        user_id: claims.sub.clone(),
        email: claims.email.clone(),
        claims,
    };

    req.extensions_mut().insert(auth_user);
    Ok(next.run(req).await)
}

/// Extract AuthUser from request extensions.
impl<S> axum::extract::FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or(AuthError::MissingAuth)
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AuthError::MissingAuth => (StatusCode::UNAUTHORIZED, "Missing authorization header"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired"),
            AuthError::InvalidToken(_) => (StatusCode::UNAUTHORIZED, "Invalid token"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Auth error"),
        };

        let body = serde_json::json!({ "error": message });
        (status, axum::Json(body)).into_response()
    }
}
