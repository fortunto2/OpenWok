use dioxus::prelude::*;
use openwok_core::types::User;

#[server]
pub async fn auth_callback(token: String) -> ServerFnResult<User> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_core::repo::Repository;
    use openwok_core::types::CreateUserRequest;

    use crate::db::repo::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;

    // Decode JWT to get supabase_user_id and email
    let jwt_secret = std::env::var("SUPABASE_JWT_SECRET")
        .map_err(|_| ServerFnError::new("SUPABASE_JWT_SECRET not configured"))?;
    let key = jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes());
    let mut validation = jsonwebtoken::Validation::default();
    validation.set_audience(&["authenticated"]);
    validation.set_required_spec_claims(&["sub", "email"]);

    let token_data = jsonwebtoken::decode::<serde_json::Value>(&token, &key, &validation)
        .map_err(|e| ServerFnError::new(format!("Invalid token: {e}")))?;

    let claims = token_data.claims;
    let sub = claims["sub"]
        .as_str()
        .ok_or_else(|| ServerFnError::new("Missing sub claim"))?;
    let email = claims["email"]
        .as_str()
        .ok_or_else(|| ServerFnError::new("Missing email claim"))?;
    let name = claims["name"].as_str().map(String::from);

    // Try to find existing user (reject if blocked)
    if let Ok(user) = repo.get_user_by_supabase_id(sub).await {
        if user.blocked {
            return Err(ServerFnError::new("User is blocked"));
        }
        return Ok(user);
    }

    // Create new user
    let req = CreateUserRequest {
        supabase_user_id: sub.to_string(),
        email: email.to_string(),
        name,
        role: None,
    };
    let user = repo
        .create_user(req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(user)
}

/// Verify JWT and return the active (non-blocked) user.
/// Reusable by any server function that needs authentication.
#[cfg(feature = "server")]
pub async fn verify_token_and_get_user(
    token: &str,
    repo: &(impl openwok_core::repo::Repository + Send + Sync),
) -> Result<User, ServerFnError> {
    let jwt_secret = std::env::var("SUPABASE_JWT_SECRET")
        .map_err(|_| ServerFnError::new("SUPABASE_JWT_SECRET not configured"))?;
    let key = jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes());
    let mut validation = jsonwebtoken::Validation::default();
    validation.set_audience(&["authenticated"]);
    validation.set_required_spec_claims(&["sub"]);

    let token_data = jsonwebtoken::decode::<serde_json::Value>(token, &key, &validation)
        .map_err(|e| ServerFnError::new(format!("Invalid token: {e}")))?;

    let sub = token_data.claims["sub"]
        .as_str()
        .ok_or_else(|| ServerFnError::new("Missing sub claim"))?;

    let user = repo
        .get_user_by_supabase_id(sub)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if user.blocked {
        return Err(ServerFnError::new("User is blocked"));
    }
    Ok(user)
}

#[server]
pub async fn get_me(token: String) -> ServerFnResult<User> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;

    use crate::db::repo::SqliteRepo;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let user = verify_token_and_get_user(&token, repo.as_ref()).await?;
    Ok(user)
}
