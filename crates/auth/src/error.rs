use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

/// Auth errors.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Environment error: {0}")]
    Env(#[from] std::env::VarError),

    #[error("Supabase error ({status}): {message}")]
    Supabase { status: u16, message: String },

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Missing authorization header")]
    MissingAuth,

    #[error("Configuration error: {0}")]
    Config(String),
}

impl AuthError {
    pub fn code(&self) -> AuthErrorCode {
        match self {
            Self::Supabase { message, .. } => parse_supabase_error_code(message),
            Self::InvalidToken(_) => AuthErrorCode::InvalidSession,
            Self::TokenExpired => AuthErrorCode::SessionExpired,
            Self::MissingAuth => AuthErrorCode::InvalidSession,
            Self::Config(_) | Self::Env(_) => AuthErrorCode::Configuration,
            Self::Request(_) => AuthErrorCode::Unknown,
        }
    }

    pub fn user_message(&self) -> &'static str {
        match self.code() {
            AuthErrorCode::InvalidCredentials => "Invalid email or password.",
            AuthErrorCode::EmailNotConfirmed => "Confirm your email address before signing in.",
            AuthErrorCode::EmailRateLimited => {
                "Too many email requests were sent. Try again in a few minutes."
            }
            AuthErrorCode::PasswordTooShort => "Use a password with at least 8 characters.",
            AuthErrorCode::InvalidEmail => "Enter a valid email address.",
            AuthErrorCode::AccountBlocked => "Your account is blocked. Contact support.",
            AuthErrorCode::MissingEmail => {
                "No email address was returned by the identity provider."
            }
            AuthErrorCode::InvalidSession => "Your session is invalid. Sign in again.",
            AuthErrorCode::SessionExpired => "Your session expired. Sign in again.",
            AuthErrorCode::Configuration => "Authentication is not configured correctly.",
            AuthErrorCode::Unknown => "Authentication failed. Please try again.",
        }
    }
}

fn parse_supabase_error_code(message: &str) -> AuthErrorCode {
    let lower = message.to_ascii_lowercase();
    let explicit_code = serde_json::from_str::<SupabaseErrorPayload>(message)
        .ok()
        .and_then(|payload| payload.error_code.or(payload.code))
        .map(|code| code.to_ascii_lowercase());

    match explicit_code.as_deref() {
        Some("invalid_credentials") => AuthErrorCode::InvalidCredentials,
        Some("email_not_confirmed") => AuthErrorCode::EmailNotConfirmed,
        Some("over_email_send_rate_limit") => AuthErrorCode::EmailRateLimited,
        Some("weak_password") => AuthErrorCode::PasswordTooShort,
        Some("email_address_invalid") => AuthErrorCode::InvalidEmail,
        Some(_) => AuthErrorCode::Unknown,
        None if lower.contains("invalid_credentials") => AuthErrorCode::InvalidCredentials,
        None if lower.contains("email_not_confirmed") || lower.contains("email not confirmed") => {
            AuthErrorCode::EmailNotConfirmed
        }
        None if lower.contains("over_email_send_rate_limit")
            || lower.contains("email rate limit") =>
        {
            AuthErrorCode::EmailRateLimited
        }
        None if lower.contains("weak_password")
            || lower.contains("password should be at least") =>
        {
            AuthErrorCode::PasswordTooShort
        }
        None if lower.contains("email_address_invalid") || lower.contains("invalid email") => {
            AuthErrorCode::InvalidEmail
        }
        _ => AuthErrorCode::Unknown,
    }
}

#[derive(Debug, Deserialize)]
struct SupabaseErrorPayload {
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    error_code: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{AuthError, AuthErrorCode};

    #[test]
    fn maps_invalid_credentials_supabase_error() {
        let err = AuthError::Supabase {
            status: 400,
            message: r#"{"code":400,"error_code":"invalid_credentials","msg":"Invalid login credentials"}"#.into(),
        };

        assert_eq!(err.code(), AuthErrorCode::InvalidCredentials);
        assert_eq!(err.user_message(), "Invalid email or password.");
    }

    #[test]
    fn maps_email_rate_limit_without_structured_json() {
        let err = AuthError::Supabase {
            status: 429,
            message: "email rate limit exceeded".into(),
        };

        assert_eq!(err.code(), AuthErrorCode::EmailRateLimited);
    }

    #[test]
    fn maps_config_to_configuration_message() {
        let err = AuthError::Config("missing SUPABASE_URL".into());

        assert_eq!(err.code(), AuthErrorCode::Configuration);
        assert_eq!(
            err.user_message(),
            "Authentication is not configured correctly."
        );
    }
}
