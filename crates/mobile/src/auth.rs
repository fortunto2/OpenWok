#![allow(non_snake_case)]

use crate::api::api_post_json;
use crate::config::{OAUTH_REDIRECT_URL, SUPABASE_URL};
use crate::state::UserState;
use crate::storage;

/// Build the Supabase Google OAuth URL.
pub fn oauth_url() -> String {
    format!("{SUPABASE_URL}/auth/v1/authorize?provider=google&redirect_to={OAUTH_REDIRECT_URL}")
}

/// Open system browser for Google sign-in.
pub fn start_oauth() {
    let url = oauth_url();
    let _ = open::that(&url);
}

/// Parse access_token from a deep link callback URL.
/// Expected format: openwok://auth/callback#access_token=xxx&token_type=bearer&...
pub fn parse_callback_token(url: &str) -> Option<String> {
    let fragment = url.split('#').nth(1)?;
    fragment.split('&').find_map(|part| {
        let (key, val) = part.split_once('=')?;
        if key == "access_token" {
            Some(val.to_string())
        } else {
            None
        }
    })
}

/// Verify token with backend and return user state.
pub async fn verify_token(token: &str) -> Result<UserState, String> {
    let body = serde_json::json!({ "access_token": token });
    let data: serde_json::Value = api_post_json("/auth/callback", &body.to_string()).await?;
    let email = data["user"]["email"].as_str().map(|s| s.to_string());
    storage::save_jwt(token);
    Ok(UserState {
        jwt: Some(token.to_string()),
        email,
    })
}

/// Logout — clear stored JWT.
pub fn logout() {
    storage::clear_jwt();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_token_from_callback_url() {
        let url = "openwok://auth/callback#access_token=abc123&token_type=bearer&expires_in=3600";
        assert_eq!(parse_callback_token(url), Some("abc123".to_string()));
    }

    #[test]
    fn parse_token_missing() {
        let url = "openwok://auth/callback#token_type=bearer";
        assert_eq!(parse_callback_token(url), None);
    }

    #[test]
    fn parse_token_no_fragment() {
        let url = "openwok://auth/callback";
        assert_eq!(parse_callback_token(url), None);
    }

    #[test]
    fn oauth_url_format() {
        let url = oauth_url();
        assert!(url.contains("provider=google"));
        assert!(url.contains("redirect_to=openwok://auth/callback"));
    }
}
