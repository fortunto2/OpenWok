use crate::config::AuthConfig;
use crate::error::AuthError;
use crate::types::{Claims, Session, User, UserProfile};
use jsonwebtoken::jwk::JwkSet;

enum TokenVerifier {
    SharedSecret,
    Jwks(JwkSet),
    Unconfigured,
}

/// Supabase Auth client for SuperDuperAi apps.
pub struct AuthClient {
    config: AuthConfig,
    http: reqwest::Client,
    verifier: TokenVerifier,
}

impl AuthClient {
    pub fn new(config: AuthConfig) -> Self {
        let verifier = if config.jwt_secret.is_some() {
            TokenVerifier::SharedSecret
        } else {
            TokenVerifier::Unconfigured
        };

        Self {
            config,
            http: reqwest::Client::new(),
            verifier,
        }
    }

    pub async fn from_config(config: AuthConfig) -> Result<Self, AuthError> {
        let http = reqwest::Client::new();
        let verifier = if config.jwt_secret.is_some() {
            TokenVerifier::SharedSecret
        } else {
            TokenVerifier::Jwks(fetch_jwks(&http, &config).await?)
        };

        Ok(Self {
            config,
            http,
            verifier,
        })
    }

    /// Sign in with Google ID token (from native Google Sign-In).
    pub async fn sign_in_with_id_token(&self, id_token: &str) -> Result<Session, AuthError> {
        let url = format!("{}/token?grant_type=id_token", self.config.auth_url());
        let body = serde_json::json!({
            "provider": "google",
            "id_token": id_token,
        });

        let resp = self
            .http
            .post(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        self.parse_session(resp).await
    }

    /// Sign in with email + password.
    pub async fn sign_in_with_password(
        &self,
        email: &str,
        password: &str,
    ) -> Result<Session, AuthError> {
        let url = format!("{}/token?grant_type=password", self.config.auth_url());
        let body = serde_json::json!({
            "email": email,
            "password": password,
        });

        let resp = self
            .http
            .post(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .json(&body)
            .send()
            .await?;

        self.parse_session(resp).await
    }

    /// Sign up with email + password.
    pub async fn sign_up(&self, email: &str, password: &str) -> Result<Session, AuthError> {
        let url = format!("{}/signup", self.config.auth_url());
        let body = serde_json::json!({
            "email": email,
            "password": password,
        });

        let resp = self
            .http
            .post(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .json(&body)
            .send()
            .await?;

        self.parse_session(resp).await
    }

    /// Sign up with email + password when email confirmation is enabled.
    ///
    /// Supabase can return a bare user object without a session in this flow.
    pub async fn sign_up_user(&self, email: &str, password: &str) -> Result<User, AuthError> {
        let url = format!("{}/signup", self.config.auth_url());
        let body = serde_json::json!({
            "email": email,
            "password": password,
        });

        let resp = self
            .http
            .post(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .json(&body)
            .send()
            .await?;

        self.parse_json(resp).await
    }

    /// Refresh session with refresh token.
    pub async fn refresh_session(&self, refresh_token: &str) -> Result<Session, AuthError> {
        let url = format!("{}/token?grant_type=refresh_token", self.config.auth_url());
        let body = serde_json::json!({
            "refresh_token": refresh_token,
        });

        let resp = self
            .http
            .post(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .json(&body)
            .send()
            .await?;

        self.parse_session(resp).await
    }

    /// Sign out (invalidate refresh token).
    pub async fn sign_out(&self, access_token: &str) -> Result<(), AuthError> {
        let url = format!("{}/logout", self.config.auth_url());

        let resp = self
            .http
            .post(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .bearer_auth(access_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(AuthError::Supabase {
                status,
                message: body,
            });
        }
        Ok(())
    }

    /// Get current user from access token.
    pub async fn get_user(&self, access_token: &str) -> Result<User, AuthError> {
        let url = format!("{}/user", self.config.auth_url());

        let resp = self
            .http
            .get(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .bearer_auth(access_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(AuthError::Supabase {
                status,
                message: body,
            });
        }

        resp.json().await.map_err(AuthError::Request)
    }

    /// Get or create app-specific profile (from profiles table via REST).
    pub async fn get_profile(&self, access_token: &str) -> Result<Option<UserProfile>, AuthError> {
        let url = format!(
            "{}/profiles?app_id=eq.{}&select=*",
            self.config.rest_url(),
            self.config.app_id
        );

        let resp = self
            .http
            .get(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .header("x-app-id", &self.config.app_id)
            .bearer_auth(access_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(AuthError::Supabase {
                status,
                message: body,
            });
        }

        let profiles: Vec<UserProfile> = resp.json().await.map_err(AuthError::Request)?;
        Ok(profiles.into_iter().next())
    }

    /// Verify JWT access token locally (no network call).
    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        match &self.verifier {
            TokenVerifier::SharedSecret => {
                let secret = self
                    .config
                    .jwt_secret
                    .as_deref()
                    .ok_or_else(|| AuthError::Config("JWT secret not configured".into()))?;

                let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
                let validation = self.validation_for(jsonwebtoken::Algorithm::HS256);

                let token_data = jsonwebtoken::decode::<Claims>(token, &key, &validation).map_err(
                    |e| match e.kind() {
                        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                            AuthError::TokenExpired
                        }
                        _ => AuthError::InvalidToken(e.to_string()),
                    },
                )?;

                Ok(token_data.claims)
            }
            TokenVerifier::Jwks(jwks) => self.verify_token_with_jwks(token, jwks),
            TokenVerifier::Unconfigured => Err(AuthError::Config(
                "JWT verifier is not configured; set SUPABASE_URL or SUPABASE_JWT_SECRET".into(),
            )),
        }
    }

    // ── OAuth (pattern from supabase-rust) ──

    /// Get OAuth sign-in URL for redirect flow.
    pub fn get_oauth_url(
        &self,
        provider: crate::types::OAuthProvider,
        options: &crate::types::OAuthOptions,
    ) -> String {
        let mut url = format!(
            "{}/authorize?provider={}",
            self.config.auth_url(),
            provider.as_str()
        );
        if let Some(redirect) = &options.redirect_to {
            url.push_str(&format!("&redirect_to={redirect}"));
        }
        if let Some(scopes) = &options.scopes {
            url.push_str(&format!("&scopes={scopes}"));
        }
        url
    }

    /// Exchange OAuth code for session (after redirect callback).
    pub async fn exchange_code_for_session(&self, code: &str) -> Result<Session, AuthError> {
        let url = format!("{}/token?grant_type=pkce", self.config.auth_url());
        let body = serde_json::json!({ "auth_code": code });
        let resp = self
            .http
            .post(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .json(&body)
            .send()
            .await?;
        self.parse_session(resp).await
    }

    // ── Password reset ──

    /// Send password reset email.
    pub async fn reset_password_for_email(&self, email: &str) -> Result<(), AuthError> {
        let url = format!("{}/recover", self.config.auth_url());
        let body = serde_json::json!({ "email": email });
        let resp = self
            .http
            .post(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .json(&body)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(AuthError::Supabase {
                status,
                message: body,
            });
        }
        Ok(())
    }

    // ── Anonymous sign-in ──

    /// Sign in anonymously (creates a temporary user).
    pub async fn sign_in_anonymously(&self) -> Result<Session, AuthError> {
        let url = format!("{}/token?grant_type=anonymous", self.config.auth_url());
        let resp = self
            .http
            .post(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .send()
            .await?;
        self.parse_session(resp).await
    }

    // ── MFA ──

    /// Enroll TOTP MFA factor.
    pub async fn enroll_totp(
        &self,
        access_token: &str,
    ) -> Result<crate::types::TotpSetupInfo, AuthError> {
        let url = format!("{}/factors", self.config.auth_url());
        let body = serde_json::json!({ "factor_type": "totp" });
        let resp = self
            .http
            .post(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .bearer_auth(access_token)
            .json(&body)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(AuthError::Supabase {
                status,
                message: body,
            });
        }
        resp.json().await.map_err(AuthError::Request)
    }

    /// List MFA factors for current user.
    pub async fn list_factors(
        &self,
        access_token: &str,
    ) -> Result<Vec<crate::types::MfaFactor>, AuthError> {
        let url = format!("{}/factors", self.config.auth_url());
        let resp = self
            .http
            .get(&url)
            .header("apikey", &self.config.supabase_anon_key)
            .bearer_auth(access_token)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(AuthError::Supabase {
                status,
                message: body,
            });
        }
        resp.json().await.map_err(AuthError::Request)
    }

    /// App ID this client is configured for.
    pub fn app_id(&self) -> &str {
        &self.config.app_id
    }

    async fn parse_session(&self, resp: reqwest::Response) -> Result<Session, AuthError> {
        self.parse_json(resp).await
    }

    async fn parse_json<T: serde::de::DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T, AuthError> {
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(AuthError::Supabase {
                status,
                message: body,
            });
        }
        resp.json().await.map_err(AuthError::Request)
    }

    fn validation_for(&self, algorithm: jsonwebtoken::Algorithm) -> jsonwebtoken::Validation {
        let mut validation = jsonwebtoken::Validation::new(algorithm);
        validation.set_audience(&["authenticated"]);
        if let Some(ref issuer) = self.config.jwt_issuer {
            validation.set_issuer(&[issuer]);
        }
        validation
    }

    fn verify_token_with_jwks(&self, token: &str, jwks: &JwkSet) -> Result<Claims, AuthError> {
        let header = jsonwebtoken::decode_header(token)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        let jwk = match header.kid.as_deref() {
            Some(kid) => jwks
                .find(kid)
                .ok_or_else(|| AuthError::InvalidToken(format!("Unknown signing key id: {kid}")))?,
            None if jwks.keys.len() == 1 => &jwks.keys[0],
            None => {
                return Err(AuthError::InvalidToken(
                    "Missing key id in token header".into(),
                ));
            }
        };

        let key = jsonwebtoken::DecodingKey::from_jwk(jwk)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        let validation = self.validation_for(header.alg);
        let token_data = jsonwebtoken::decode::<Claims>(token, &key, &validation).map_err(|e| {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken(e.to_string()),
            }
        })?;

        Ok(token_data.claims)
    }
}

async fn fetch_jwks(http: &reqwest::Client, config: &AuthConfig) -> Result<JwkSet, AuthError> {
    if config.supabase_url.is_empty() {
        return Err(AuthError::Config(
            "SUPABASE_URL must be set when using JWKS verification".into(),
        ));
    }

    let response = http.get(config.jwks_url()).send().await?;
    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(AuthError::Supabase {
            status,
            message: body,
        });
    }

    let jwks: JwkSet = response.json().await.map_err(AuthError::Request)?;
    if jwks.keys.is_empty() {
        return Err(AuthError::Config(
            "Supabase JWKS endpoint returned no keys".into(),
        ));
    }

    Ok(jwks)
}

#[cfg(test)]
mod tests {
    use super::*;

    use jsonwebtoken::{EncodingKey, Header};

    fn claims(issuer: Option<&str>) -> Claims {
        Claims {
            sub: "user-123".into(),
            email: Some("user@example.com".into()),
            role: Some("authenticated".into()),
            aud: Some("authenticated".into()),
            iss: issuer.map(str::to_string),
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: Some(chrono::Utc::now().timestamp()),
            app_metadata: serde_json::json!({}),
            user_metadata: serde_json::json!({}),
        }
    }

    fn encode_token(claims: &Claims, secret: &str) -> String {
        let key = EncodingKey::from_secret(secret.as_bytes());
        jsonwebtoken::encode(&Header::default(), claims, &key).unwrap()
    }

    #[test]
    fn verify_token_accepts_matching_issuer() {
        let auth = AuthClient::new(
            AuthConfig::server_only("openwok", "test-secret")
                .with_jwt_issuer("https://issuer.example/auth/v1"),
        );
        let token = encode_token(
            &claims(Some("https://issuer.example/auth/v1")),
            "test-secret",
        );

        let parsed = auth.verify_token(&token).unwrap();
        assert_eq!(parsed.sub, "user-123");
    }

    #[test]
    fn verify_token_rejects_mismatched_issuer() {
        let auth = AuthClient::new(
            AuthConfig::server_only("openwok", "test-secret")
                .with_jwt_issuer("https://issuer.example/auth/v1"),
        );
        let token = encode_token(
            &claims(Some("https://wrong.example/auth/v1")),
            "test-secret",
        );

        let error = auth.verify_token(&token).unwrap_err();
        assert!(matches!(error, AuthError::InvalidToken(_)));
    }
}
