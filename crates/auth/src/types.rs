use serde::{Deserialize, Serialize};

/// Supabase auth session (access + refresh tokens).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub expires_at: Option<i64>,
    pub user: User,
}

/// Supabase user from auth.users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub role: Option<String>,
    pub confirmed_at: Option<String>,
    pub last_sign_in_at: Option<String>,
    #[serde(default)]
    pub identities: Vec<serde_json::Value>,
    #[serde(default)]
    pub app_metadata: serde_json::Value,
    #[serde(default)]
    pub user_metadata: serde_json::Value,
}

/// App-specific user profile (from profiles table).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub app_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// JWT claims from Supabase access token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    pub role: Option<String>,
    pub aud: Option<String>,
    pub iss: Option<String>,
    pub exp: i64,
    pub iat: Option<i64>,
    #[serde(default)]
    pub app_metadata: serde_json::Value,
    #[serde(default)]
    pub user_metadata: serde_json::Value,
}

/// OAuth providers supported by Supabase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OAuthProvider {
    Google,
    Github,
    Gitlab,
    Facebook,
    Apple,
    Twitter,
    Discord,
    Slack,
    Spotify,
    Twitch,
    Azure,
    Bitbucket,
    Linkedin,
    Notion,
    Figma,
    Zoom,
}

impl OAuthProvider {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Google => "google",
            Self::Github => "github",
            Self::Gitlab => "gitlab",
            Self::Facebook => "facebook",
            Self::Apple => "apple",
            Self::Twitter => "twitter",
            Self::Discord => "discord",
            Self::Slack => "slack",
            Self::Spotify => "spotify",
            Self::Twitch => "twitch",
            Self::Azure => "azure",
            Self::Bitbucket => "bitbucket",
            Self::Linkedin => "linkedin_oidc",
            Self::Notion => "notion",
            Self::Figma => "figma",
            Self::Zoom => "zoom",
        }
    }
}

/// OAuth sign-in options.
#[derive(Debug, Clone, Default)]
pub struct OAuthOptions {
    /// Redirect URL after OAuth flow.
    pub redirect_to: Option<String>,
    /// OAuth scopes to request.
    pub scopes: Option<String>,
    /// Additional query params.
    pub query_params: Option<Vec<(String, String)>>,
}

/// MFA factor types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MfaFactorType {
    Totp,
    Phone,
}

/// MFA factor status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MfaFactorStatus {
    Unverified,
    Verified,
}

/// MFA factor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaFactor {
    pub id: String,
    pub factor_type: MfaFactorType,
    pub status: MfaFactorStatus,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// TOTP setup info (QR code + secret).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSetupInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub factor_type: String,
    pub totp: TotpDetails,
}

/// TOTP details for enrollment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpDetails {
    pub qr_code: String,
    pub secret: String,
    pub uri: String,
}
