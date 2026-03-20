use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use openwok_core::repo::Repository;
use openwok_core::types::{
    Dispute, ResolveDisputeRequest, SetBlockedRequest, User, UserId, UserRole,
};

use crate::auth::AuthUser;
use crate::restaurants::repo_error_to_status;

/// Verify that the authenticated user is a non-blocked NodeOperator.
/// Returns the User on success, or a 403 error.
async fn require_admin<R: Repository>(
    repo: &R,
    auth: &AuthUser,
) -> Result<User, (StatusCode, String)> {
    let user = repo
        .get_user_by_supabase_id(&auth.supabase_user_id)
        .await
        .map_err(|_| (StatusCode::FORBIDDEN, "user not found".into()))?;

    if user.blocked {
        return Err((StatusCode::FORBIDDEN, "user is blocked".into()));
    }
    if user.role != UserRole::NodeOperator {
        return Err((StatusCode::FORBIDDEN, "admin access required".into()));
    }
    Ok(user)
}

/// Helper to load an active (non-blocked) user from auth.
/// Returns 403 if blocked.
pub async fn get_active_user<R: Repository>(
    repo: &R,
    auth: &AuthUser,
) -> Result<User, (StatusCode, String)> {
    let user = repo
        .get_user_by_supabase_id(&auth.supabase_user_id)
        .await
        .map_err(|_| (StatusCode::FORBIDDEN, "user not found".into()))?;

    if user.blocked {
        return Err((StatusCode::FORBIDDEN, "user is blocked".into()));
    }
    Ok(user)
}

/// GET /admin/users — list all users (admin only)
#[utoipa::path(get, path = "/admin/users", tag = "admin")]
pub async fn list_users<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    require_admin(repo.as_ref(), &auth).await?;
    repo.list_users()
        .await
        .map(Json)
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))
}

/// PATCH /admin/users/{id}/block — toggle blocked status (admin only)
#[utoipa::path(patch, path = "/admin/users/{id}/block", tag = "admin")]
pub async fn toggle_block<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
    Path(id): Path<UserId>,
    Json(body): Json<SetBlockedRequest>,
) -> Result<Json<User>, (StatusCode, String)> {
    require_admin(repo.as_ref(), &auth).await?;
    repo.set_user_blocked(id, body.blocked)
        .await
        .map(Json)
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))
}

/// GET /admin/disputes — list all disputes (admin only)
#[utoipa::path(get, path = "/admin/disputes", tag = "admin")]
pub async fn list_disputes<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
) -> Result<Json<Vec<Dispute>>, (StatusCode, String)> {
    require_admin(repo.as_ref(), &auth).await?;
    repo.list_disputes()
        .await
        .map(Json)
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))
}

/// PATCH /admin/disputes/{id}/resolve — resolve or dismiss a dispute (admin only)
#[utoipa::path(patch, path = "/admin/disputes/{id}/resolve", tag = "admin")]
pub async fn resolve_dispute<R: Repository>(
    auth: AuthUser,
    State(repo): State<Arc<R>>,
    Path(id): Path<openwok_core::types::DisputeId>,
    Json(body): Json<ResolveDisputeRequest>,
) -> Result<Json<Dispute>, (StatusCode, String)> {
    require_admin(repo.as_ref(), &auth).await?;
    repo.resolve_dispute(id, body.status, body.resolution)
        .await
        .map(Json)
        .map_err(|e| (repo_error_to_status(&e), e.to_string()))
}
