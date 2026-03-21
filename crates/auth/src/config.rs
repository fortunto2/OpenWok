/// Configuration for SuperDuperAi Auth.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Supabase project URL (e.g., "https://xxx.supabase.co")
    pub supabase_url: String,
    /// Supabase anon key
    pub supabase_anon_key: String,
    /// App identifier for RLS (e.g., "openwok", "supervox", "facealarm")
    pub app_id: String,
    /// JWT secret for local verification (optional — fetches JWKS if not set)
    pub jwt_secret: Option<String>,
    /// Expected JWT issuer for local verification.
    pub jwt_issuer: Option<String>,
}

impl AuthConfig {
    pub fn new(supabase_url: &str, anon_key: &str, app_id: &str) -> Self {
        Self {
            supabase_url: supabase_url.to_string(),
            supabase_anon_key: anon_key.to_string(),
            app_id: app_id.to_string(),
            jwt_secret: None,
            jwt_issuer: None,
        }
    }

    /// Build a verification-only config for server-side JWT validation.
    pub fn server_only(app_id: &str, jwt_secret: &str) -> Self {
        Self {
            supabase_url: String::new(),
            supabase_anon_key: String::new(),
            app_id: app_id.to_string(),
            jwt_secret: Some(jwt_secret.to_string()),
            jwt_issuer: None,
        }
    }

    /// Load from environment variables:
    /// - `SUPABASE_URL`
    /// - `SUPABASE_ANON_KEY`
    /// - `APP_ID`
    /// - `SUPABASE_JWT_SECRET` (optional)
    /// - `SUPABASE_JWT_ISSUER` (optional)
    pub fn from_env(app_id: &str) -> Result<Self, crate::error::AuthError> {
        let supabase_url = std::env::var("SUPABASE_URL")?;
        let config = Self {
            jwt_issuer: std::env::var("SUPABASE_JWT_ISSUER")
                .ok()
                .or_else(|| Some(format!("{}/auth/v1", supabase_url.trim_end_matches('/')))),
            supabase_url,
            supabase_anon_key: std::env::var("SUPABASE_ANON_KEY")
                .or_else(|_| std::env::var("SUPABASE_PUBLISHABLE_KEY"))
                .unwrap_or_default(),
            app_id: app_id.to_string(),
            jwt_secret: std::env::var("SUPABASE_JWT_SECRET").ok(),
        };
        config.validate_for_client()?;
        Ok(config)
    }

    /// Load only the server-side JWT verification settings from the environment.
    pub fn from_server_env(app_id: &str) -> Result<Self, crate::error::AuthError> {
        let supabase_url = std::env::var("SUPABASE_URL")?;
        let config = Self {
            jwt_issuer: std::env::var("SUPABASE_JWT_ISSUER")
                .ok()
                .or_else(|| Some(format!("{}/auth/v1", supabase_url.trim_end_matches('/')))),
            supabase_url,
            supabase_anon_key: std::env::var("SUPABASE_ANON_KEY")
                .or_else(|_| std::env::var("SUPABASE_PUBLISHABLE_KEY"))
                .unwrap_or_default(),
            app_id: app_id.to_string(),
            jwt_secret: std::env::var("SUPABASE_JWT_SECRET").ok(),
        };
        config.validate_for_server()?;
        Ok(config)
    }

    pub fn with_jwt_secret(mut self, secret: &str) -> Self {
        self.jwt_secret = Some(secret.to_string());
        self
    }

    pub fn with_jwt_issuer(mut self, issuer: &str) -> Self {
        self.jwt_issuer = Some(issuer.to_string());
        self
    }

    /// GoTrue API base URL.
    pub(crate) fn auth_url(&self) -> String {
        format!("{}/auth/v1", self.supabase_url)
    }

    /// REST API base URL.
    pub(crate) fn rest_url(&self) -> String {
        format!("{}/rest/v1", self.supabase_url)
    }

    /// JWK set URL for verifying asymmetric Supabase access tokens.
    pub(crate) fn jwks_url(&self) -> String {
        format!(
            "{}/auth/v1/.well-known/jwks.json",
            self.supabase_url.trim_end_matches('/')
        )
    }

    pub fn validate_for_client(&self) -> Result<(), crate::error::AuthError> {
        if self.supabase_url.trim().is_empty() {
            return Err(crate::error::AuthError::Config(
                "SUPABASE_URL must be set for auth client operations".into(),
            ));
        }
        if self.supabase_anon_key.trim().is_empty() {
            return Err(crate::error::AuthError::Config(
                "SUPABASE_ANON_KEY or SUPABASE_PUBLISHABLE_KEY must be set".into(),
            ));
        }
        if self.app_id.trim().is_empty() {
            return Err(crate::error::AuthError::Config(
                "APP_ID must not be empty".into(),
            ));
        }
        Ok(())
    }

    pub fn validate_for_server(&self) -> Result<(), crate::error::AuthError> {
        if self.app_id.trim().is_empty() {
            return Err(crate::error::AuthError::Config(
                "APP_ID must not be empty".into(),
            ));
        }
        if self.jwt_secret.is_some() {
            return Ok(());
        }
        if self.supabase_url.trim().is_empty() {
            return Err(crate::error::AuthError::Config(
                "SUPABASE_URL must be set when using JWKS verification".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AuthConfig;
    use crate::error::AuthError;

    #[test]
    fn validate_for_client_requires_publishable_key() {
        let err = AuthConfig::new("https://example.supabase.co", "", "openwok")
            .validate_for_client()
            .unwrap_err();

        assert!(matches!(err, AuthError::Config(_)));
    }

    #[test]
    fn validate_for_server_accepts_shared_secret_without_supabase_url() {
        AuthConfig::server_only("openwok", "secret")
            .validate_for_server()
            .unwrap();
    }

    #[test]
    fn validate_for_server_accepts_jwks_setup() {
        AuthConfig::new("https://example.supabase.co", "sb_publishable_x", "openwok")
            .validate_for_server()
            .unwrap();
    }
}
