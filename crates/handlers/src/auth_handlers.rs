use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use openwok_core::repo::Repository;
use openwok_core::types::{CreateUserRequest, User};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::{AuthUser, JwtConfig, verify_jwt};

#[derive(Deserialize, ToSchema)]
pub struct AuthCallback {
    /// The JWT access_token from Supabase Auth
    pub access_token: String,
}

#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    pub user: User,
    /// Echo the token back so frontend can store it
    pub access_token: String,
}

/// POST /api/auth/callback
/// Receives Supabase JWT, verifies it, creates or retrieves user, returns profile.
#[utoipa::path(post, path = "/auth/callback", tag = "auth")]
pub async fn callback<R: Repository>(
    State(repo): State<Arc<R>>,
    Extension(jwt_config): Extension<JwtConfig>,
    Json(body): Json<AuthCallback>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    // Verify the JWT
    let claims = verify_jwt(&body.access_token, &jwt_config)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    // Try to find existing user by Supabase ID
    let user = match repo.get_user_by_supabase_id(&claims.sub).await {
        Ok(user) => user,
        Err(_) => {
            // Create new user on first login
            let req = CreateUserRequest {
                supabase_user_id: claims.sub,
                email: claims.email.unwrap_or_default(),
                name: None,
                role: None,
            };
            repo.create_user(req)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        }
    };

    Ok(Json(AuthResponse {
        user,
        access_token: body.access_token,
    }))
}

/// GET /api/auth/me
/// Returns the current authenticated user's profile.
#[utoipa::path(get, path = "/auth/me", tag = "auth")]
pub async fn me<R: Repository>(
    State(repo): State<Arc<R>>,
    auth: AuthUser,
) -> Result<Json<User>, (StatusCode, String)> {
    repo.get_user_by_supabase_id(&auth.supabase_user_id)
        .await
        .map(Json)
        .map_err(|_| (StatusCode::NOT_FOUND, "user not found".into()))
}
