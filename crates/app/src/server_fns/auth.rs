use dioxus::prelude::*;
use openwok_core::types::User;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PublicAuthSettings {
    pub supabase_url: String,
    pub app_url: String,
    pub google_oauth_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AuthErrorCode {
    InvalidCredentials,
    EmailNotConfirmed,
    EmailRateLimited,
    PasswordTooShort,
    InvalidEmail,
    AccountBlocked,
    MissingEmail,
    InvalidSession,
    SessionExpired,
    Configuration,
    Unknown,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AuthErrorInfo {
    pub code: AuthErrorCode,
    pub message: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AuthSessionResult {
    pub token: Option<String>,
    pub user: Option<User>,
    pub error: Option<AuthErrorInfo>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SignUpResult {
    pub email: Option<String>,
    pub confirmation_required: bool,
    pub error: Option<AuthErrorInfo>,
}

#[server]
pub async fn get_auth_settings() -> ServerFnResult<PublicAuthSettings> {
    Ok(public_auth_settings_from_env())
}

#[server]
pub async fn auth_callback(token: String) -> ServerFnResult<User> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_api::SqliteRepo;
    use superduperai_auth::AuthClient;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let Extension(auth) = FullstackContext::extract::<Extension<Arc<AuthClient>>, _>().await?;

    let claims = auth
        .verify_token(&token)
        .map_err(|e| ServerFnError::new(format!("Invalid token: {e}")))?;
    ensure_local_user_from_claims(repo.as_ref(), &claims).await
}

/// Verify JWT and return the active (non-blocked) user.
/// Reusable by any server function that needs authentication.
#[cfg(feature = "server")]
pub async fn verify_token_and_get_user(
    token: &str,
    repo: &(impl openwok_core::repo::Repository + Send + Sync),
) -> Result<User, ServerFnError> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use superduperai_auth::AuthClient;

    let Extension(auth) = FullstackContext::extract::<Extension<Arc<AuthClient>>, _>().await?;
    verify_token_and_get_user_with_auth(token, repo, auth.as_ref()).await
}

/// Verify JWT and return the active (non-blocked) user using an explicit auth client.
#[cfg(feature = "server")]
pub async fn verify_token_and_get_user_with_auth(
    token: &str,
    repo: &(impl openwok_core::repo::Repository + Send + Sync),
    auth: &superduperai_auth::AuthClient,
) -> Result<User, ServerFnError> {
    let claims = auth
        .verify_token(token)
        .map_err(|e| ServerFnError::new(format!("Invalid token: {e}")))?;

    let user = repo
        .get_user_by_supabase_id(&claims.sub)
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

    use openwok_api::SqliteRepo;
    use superduperai_auth::AuthClient;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let Extension(auth) = FullstackContext::extract::<Extension<Arc<AuthClient>>, _>().await?;
    let user = verify_token_and_get_user_with_auth(&token, repo.as_ref(), auth.as_ref()).await?;
    Ok(user)
}

#[server]
pub async fn sign_in_with_email_password(
    email: String,
    password: String,
) -> ServerFnResult<AuthSessionResult> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use openwok_api::SqliteRepo;
    use superduperai_auth::AuthClient;

    let Extension(repo) = FullstackContext::extract::<Extension<Arc<SqliteRepo>>, _>().await?;
    let Extension(auth) = FullstackContext::extract::<Extension<Arc<AuthClient>>, _>().await?;

    let session = match auth.sign_in_with_password(&email, &password).await {
        Ok(session) => session,
        Err(err) => {
            return Ok(AuthSessionResult {
                token: None,
                user: None,
                error: Some(auth_error_info_from_auth_error(&err)),
            });
        }
    };
    let claims = match auth.verify_token(&session.access_token) {
        Ok(claims) => claims,
        Err(err) => {
            return Ok(AuthSessionResult {
                token: None,
                user: None,
                error: Some(auth_error_info_from_auth_error(&err)),
            });
        }
    };
    let user = match ensure_local_user_from_claims(repo.as_ref(), &claims).await {
        Ok(user) => user,
        Err(err) => {
            return Ok(AuthSessionResult {
                token: None,
                user: None,
                error: Some(auth_error_info_from_server_error(&err)),
            });
        }
    };

    Ok(AuthSessionResult {
        token: Some(session.access_token),
        user: Some(user),
        error: None,
    })
}

#[server]
pub async fn sign_up_with_email_password(
    email: String,
    password: String,
) -> ServerFnResult<SignUpResult> {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::fullstack::FullstackContext;
    use superduperai_auth::AuthClient;

    let Extension(auth) = FullstackContext::extract::<Extension<Arc<AuthClient>>, _>().await?;

    let created = match auth.sign_up_user(&email, &password).await {
        Ok(created) => created,
        Err(err) => {
            return Ok(SignUpResult {
                email: Some(email),
                confirmation_required: false,
                error: Some(auth_error_info_from_auth_error(&err)),
            });
        }
    };

    Ok(SignUpResult {
        email: created.email.or(Some(email)),
        confirmation_required: created.confirmed_at.is_none(),
        error: None,
    })
}

#[cfg(feature = "server")]
async fn ensure_local_user_from_claims(
    repo: &(impl openwok_core::repo::Repository + Send + Sync),
    claims: &superduperai_auth::Claims,
) -> Result<User, ServerFnError> {
    use openwok_core::types::CreateUserRequest;

    let name = claims
        .user_metadata
        .get("name")
        .or_else(|| claims.app_metadata.get("name"))
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let email = claims
        .email
        .clone()
        .ok_or_else(|| ServerFnError::new("Missing email claim"))?;

    if let Ok(user) = repo.get_user_by_supabase_id(&claims.sub).await {
        if user.blocked {
            return Err(ServerFnError::new("User is blocked"));
        }
        return Ok(user);
    }

    let req = CreateUserRequest {
        supabase_user_id: claims.sub.clone(),
        email,
        name,
        role: None,
    };
    repo.create_user(req)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[cfg_attr(not(feature = "server"), allow(dead_code))]
fn public_auth_settings_from_env() -> PublicAuthSettings {
    PublicAuthSettings {
        app_url: std::env::var("APP_BASE_URL")
            .or_else(|_| std::env::var("PUBLIC_APP_URL"))
            .unwrap_or_default(),
        supabase_url: std::env::var("SUPABASE_URL").unwrap_or_default(),
        google_oauth_enabled: std::env::var("SUPABASE_GOOGLE_AUTH_ENABLED")
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
            .unwrap_or(false),
    }
}

#[cfg(feature = "server")]
fn auth_error_info_from_auth_error(error: &superduperai_auth::AuthError) -> AuthErrorInfo {
    AuthErrorInfo {
        code: map_auth_error_code(error.code()),
        message: error.user_message().to_string(),
    }
}

#[cfg_attr(not(feature = "server"), allow(dead_code))]
fn auth_error_info_from_server_error(error: &ServerFnError) -> AuthErrorInfo {
    let text = error.to_string();
    let (code, message) = if text.contains("User is blocked") {
        (
            AuthErrorCode::AccountBlocked,
            "Your account is blocked. Contact support.",
        )
    } else if text.contains("Missing email claim") {
        (
            AuthErrorCode::MissingEmail,
            "No email address was returned by the identity provider.",
        )
    } else if text.contains("Invalid token") {
        (
            AuthErrorCode::InvalidSession,
            "Your session is invalid. Sign in again.",
        )
    } else if text.contains("Token expired") {
        (
            AuthErrorCode::SessionExpired,
            "Your session expired. Sign in again.",
        )
    } else if text.to_ascii_lowercase().contains("config") {
        (
            AuthErrorCode::Configuration,
            "Authentication is not configured correctly.",
        )
    } else {
        (
            AuthErrorCode::Unknown,
            "Authentication failed. Please try again.",
        )
    };

    AuthErrorInfo {
        code,
        message: String::from(message),
    }
}

#[cfg(feature = "server")]
fn map_auth_error_code(code: superduperai_auth::AuthErrorCode) -> AuthErrorCode {
    match code {
        superduperai_auth::AuthErrorCode::InvalidCredentials => AuthErrorCode::InvalidCredentials,
        superduperai_auth::AuthErrorCode::EmailNotConfirmed => AuthErrorCode::EmailNotConfirmed,
        superduperai_auth::AuthErrorCode::EmailRateLimited => AuthErrorCode::EmailRateLimited,
        superduperai_auth::AuthErrorCode::PasswordTooShort => AuthErrorCode::PasswordTooShort,
        superduperai_auth::AuthErrorCode::InvalidEmail => AuthErrorCode::InvalidEmail,
        superduperai_auth::AuthErrorCode::AccountBlocked => AuthErrorCode::AccountBlocked,
        superduperai_auth::AuthErrorCode::MissingEmail => AuthErrorCode::MissingEmail,
        superduperai_auth::AuthErrorCode::InvalidSession => AuthErrorCode::InvalidSession,
        superduperai_auth::AuthErrorCode::SessionExpired => AuthErrorCode::SessionExpired,
        superduperai_auth::AuthErrorCode::Configuration => AuthErrorCode::Configuration,
        superduperai_auth::AuthErrorCode::Unknown => AuthErrorCode::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn maps_supabase_invalid_credentials_to_friendly_error() {
        let error = superduperai_auth::AuthError::Supabase {
            status: 400,
            message: r#"{"code":400,"error_code":"invalid_credentials","msg":"Invalid login credentials"}"#.into(),
        };

        let info = auth_error_info_from_auth_error(&error);
        assert_eq!(info.code, AuthErrorCode::InvalidCredentials);
        assert_eq!(info.message, "Invalid email or password.");
    }

    #[test]
    fn maps_blocked_user_server_error_to_friendly_error() {
        let info = auth_error_info_from_server_error(&ServerFnError::new("User is blocked"));

        assert_eq!(info.code, AuthErrorCode::AccountBlocked);
        assert_eq!(info.message, "Your account is blocked. Contact support.");
    }

    #[test]
    fn reads_public_auth_settings_from_environment() {
        let _guard = ENV_LOCK.lock().unwrap();
        let prev_app_base_url = std::env::var("APP_BASE_URL").ok();
        let prev_public_app_url = std::env::var("PUBLIC_APP_URL").ok();
        let prev_supabase_url = std::env::var("SUPABASE_URL").ok();
        let prev_google = std::env::var("SUPABASE_GOOGLE_AUTH_ENABLED").ok();

        unsafe {
            std::env::set_var("APP_BASE_URL", "https://openwok.example");
            std::env::remove_var("PUBLIC_APP_URL");
            std::env::set_var("SUPABASE_URL", "https://example.supabase.co");
            std::env::set_var("SUPABASE_GOOGLE_AUTH_ENABLED", "true");
        }

        let settings = public_auth_settings_from_env();
        assert_eq!(settings.app_url, "https://openwok.example");
        assert_eq!(settings.supabase_url, "https://example.supabase.co");
        assert!(settings.google_oauth_enabled);

        unsafe {
            restore_env_var("APP_BASE_URL", prev_app_base_url);
            restore_env_var("PUBLIC_APP_URL", prev_public_app_url);
            restore_env_var("SUPABASE_URL", prev_supabase_url);
            restore_env_var("SUPABASE_GOOGLE_AUTH_ENABLED", prev_google);
        }
    }

    unsafe fn restore_env_var(key: &str, value: Option<String>) {
        match value {
            Some(value) => unsafe { std::env::set_var(key, value) },
            None => unsafe { std::env::remove_var(key) },
        }
    }
}
