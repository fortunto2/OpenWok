use axum::extract::{Extension, FromRequestParts};
use axum::http::StatusCode;
use axum::http::request::Parts;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

/// Configuration for Supabase JWT verification.
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: Option<String>,
}

/// Supabase JWT claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseClaims {
    pub sub: String,
    pub email: Option<String>,
    pub role: Option<String>,
    pub exp: u64,
    pub iat: Option<u64>,
    pub aud: Option<String>,
    pub iss: Option<String>,
}

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
            AuthError::InvalidFormat => write!(f, "invalid Authorization format, expected: Bearer <token>"),
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
pub fn verify_jwt(token: &str, config: &JwtConfig) -> Result<SupabaseClaims, AuthError> {
    let key = DecodingKey::from_secret(config.secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);

    if let Some(ref issuer) = config.issuer {
        validation.set_issuer(&[issuer]);
    } else {
        validation.validate_aud = false;
    }
    // Supabase tokens use "authenticated" as audience
    validation.set_audience(&["authenticated"]);

    jsonwebtoken::decode::<SupabaseClaims>(token, &key, &validation)
        .map(|data| data.claims)
        .map_err(|e| AuthError::InvalidToken(e.to_string()))
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
        // Extract JwtConfig from Extension (set by the api/worker layer)
        let Extension(config) =
            axum::extract::Extension::<JwtConfig>::from_request_parts(parts, state)
                .await
                .map_err(|_| AuthError::InvalidToken("JWT config not available".into()))?;

        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthError::MissingHeader)?;

        let token = extract_bearer(auth_header)?;
        let claims = verify_jwt(token, &config)?;

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

    fn test_config() -> JwtConfig {
        JwtConfig {
            secret: "super-secret-jwt-token-for-testing-only".into(),
            issuer: None,
        }
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
            exp: chrono::Utc::now().timestamp() as u64 + 3600,
            iat: Some(chrono::Utc::now().timestamp() as u64),
            aud: Some("authenticated".into()),
            iss: None,
        }
    }

    #[test]
    fn verify_valid_token() {
        let config = test_config();
        let claims = valid_claims();
        let token = make_token(&claims, &config.secret);

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
        claims.exp = 1000; // expired long ago

        let token = make_token(&claims, &config.secret);
        let result = verify_jwt(&token, &config);
        assert!(result.is_err());
        match result {
            Err(AuthError::InvalidToken(msg)) => {
                assert!(msg.contains("ExpiredSignature"));
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
        let config = JwtConfig {
            secret: "test-secret".into(),
            issuer: Some("https://my.supabase.co/auth/v1".into()),
        };
        let mut claims = valid_claims();
        claims.iss = Some("https://wrong-issuer.com".into());
        let token = make_token(&claims, &config.secret);
        let result = verify_jwt(&token, &config);
        assert!(result.is_err());
    }

    #[test]
    fn verify_with_issuer_match() {
        let config = JwtConfig {
            secret: "test-secret".into(),
            issuer: Some("https://my.supabase.co/auth/v1".into()),
        };
        let mut claims = valid_claims();
        claims.iss = Some("https://my.supabase.co/auth/v1".into());
        let token = make_token(&claims, &config.secret);
        let result = verify_jwt(&token, &config);
        assert!(result.is_ok());
    }
}
