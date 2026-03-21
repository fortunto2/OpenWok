use std::sync::Arc;

use axum::extract::{Extension, FromRequestParts};
use axum::http::StatusCode;
use axum::http::request::Parts;
use superduperai_auth::{AuthClient, AuthError as SharedAuthError, Claims};

pub type JwtConfig = Arc<AuthClient>;
pub type SupabaseClaims = Claims;

/// Authenticated user extracted from JWT.
/// Use as an axum extractor on protected routes.
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// Supabase user UUID (from `sub` claim)
    pub supabase_user_id: String,
    /// User email (from `email` claim)
    pub email: Option<String>,
}

/// Error type for auth extraction failures.
#[derive(Debug)]
pub enum AuthError {
    MissingHeader,
    InvalidFormat,
    InvalidToken(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::MissingHeader => write!(f, "missing Authorization header"),
            AuthError::InvalidFormat => {
                write!(f, "invalid Authorization format, expected: Bearer <token>")
            }
            AuthError::InvalidToken(msg) => write!(f, "invalid token: {msg}"),
        }
    }
}

impl axum::response::IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            AuthError::MissingHeader | AuthError::InvalidFormat => StatusCode::UNAUTHORIZED,
            AuthError::InvalidToken(_) => StatusCode::UNAUTHORIZED,
        };
        (status, self.to_string()).into_response()
    }
}

/// Verify a JWT token string and extract claims.
pub fn verify_jwt(token: &str, auth: &JwtConfig) -> Result<SupabaseClaims, AuthError> {
    auth.verify_token(token).map_err(map_shared_auth_error)
}

fn map_shared_auth_error(error: SharedAuthError) -> AuthError {
    match error {
        SharedAuthError::MissingAuth => AuthError::MissingHeader,
        SharedAuthError::TokenExpired => AuthError::InvalidToken("Token expired".into()),
        SharedAuthError::InvalidToken(message) => AuthError::InvalidToken(message),
        other => AuthError::InvalidToken(other.to_string()),
    }
}

/// Extract Bearer token from Authorization header value.
fn extract_bearer(header_value: &str) -> Result<&str, AuthError> {
    header_value
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidFormat)
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(auth) =
            axum::extract::Extension::<JwtConfig>::from_request_parts(parts, state)
                .await
                .map_err(|_| AuthError::InvalidToken("JWT config not available".into()))?;

        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthError::MissingHeader)?;

        let token = extract_bearer(auth_header)?;
        let claims = verify_jwt(token, &auth)?;

        Ok(AuthUser {
            supabase_user_id: claims.sub,
            email: claims.email,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use jsonwebtoken::{EncodingKey, Header};
    use superduperai_auth::AuthConfig;

    fn test_config() -> JwtConfig {
        Arc::new(AuthClient::new(AuthConfig::server_only(
            "openwok",
            "super-secret-jwt-token-for-testing-only",
        )))
    }

    fn test_config_with_issuer(issuer: &str) -> JwtConfig {
        Arc::new(AuthClient::new(
            AuthConfig::server_only("openwok", "test-secret").with_jwt_issuer(issuer),
        ))
    }

    fn make_token(claims: &SupabaseClaims, secret: &str) -> String {
        let key = EncodingKey::from_secret(secret.as_bytes());
        jsonwebtoken::encode(&Header::default(), claims, &key).unwrap()
    }

    fn valid_claims() -> SupabaseClaims {
        SupabaseClaims {
            sub: "user-uuid-123".into(),
            email: Some("test@example.com".into()),
            role: Some("authenticated".into()),
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: Some(chrono::Utc::now().timestamp()),
            aud: Some("authenticated".into()),
            iss: None,
            app_metadata: serde_json::json!({}),
            user_metadata: serde_json::json!({}),
        }
    }

    #[test]
    fn verify_valid_token() {
        let config = test_config();
        let claims = valid_claims();
        let token = make_token(&claims, "super-secret-jwt-token-for-testing-only");

        let result = verify_jwt(&token, &config);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.sub, "user-uuid-123");
        assert_eq!(parsed.email, Some("test@example.com".into()));
    }

    #[test]
    fn verify_expired_token() {
        let config = test_config();
        let mut claims = valid_claims();
        claims.exp = 1000;

        let token = make_token(&claims, "super-secret-jwt-token-for-testing-only");
        let result = verify_jwt(&token, &config);
        assert!(result.is_err());
        match result {
            Err(AuthError::InvalidToken(msg)) => {
                assert!(msg.contains("Token expired"));
            }
            _ => panic!("expected InvalidToken error"),
        }
    }

    #[test]
    fn verify_wrong_secret() {
        let config = test_config();
        let claims = valid_claims();
        let token = make_token(&claims, "wrong-secret");

        let result = verify_jwt(&token, &config);
        assert!(result.is_err());
        match result {
            Err(AuthError::InvalidToken(msg)) => {
                assert!(msg.contains("InvalidSignature"));
            }
            _ => panic!("expected InvalidToken error"),
        }
    }

    #[test]
    fn verify_malformed_token() {
        let config = test_config();
        let result = verify_jwt("not.a.valid.token", &config);
        assert!(result.is_err());
    }

    #[test]
    fn extract_bearer_valid() {
        let result = extract_bearer("Bearer abc123");
        assert_eq!(result.unwrap(), "abc123");
    }

    #[test]
    fn extract_bearer_missing_prefix() {
        let result = extract_bearer("Basic abc123");
        assert!(result.is_err());
    }

    #[test]
    fn verify_with_issuer_mismatch() {
        let config = test_config_with_issuer("https://my.supabase.co/auth/v1");
        let mut claims = valid_claims();
        claims.iss = Some("https://wrong-issuer.com".into());
        let token = make_token(&claims, "test-secret");
        let result = verify_jwt(&token, &config);
        assert!(result.is_err());
    }

    #[test]
    fn verify_with_issuer_match() {
        let config = test_config_with_issuer("https://my.supabase.co/auth/v1");
        let mut claims = valid_claims();
        claims.iss = Some("https://my.supabase.co/auth/v1".into());
        let token = make_token(&claims, "test-secret");
        let result = verify_jwt(&token, &config);
        assert!(result.is_ok());
    }
}
