#![allow(non_snake_case)]

/// API base URL — absolute, since mobile app has no same-origin server.
pub const API_BASE: &str = "https://openwok.superduperai.co/api";

/// Supabase project URL for Google OAuth.
pub const SUPABASE_URL: &str = "https://eknpyxhulcjfmgkxfxsv.supabase.co";

/// Supabase anon key (public, safe to embed in client).
pub const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.placeholder";

/// Deep link scheme for OAuth callback.
pub const DEEP_LINK_SCHEME: &str = "openwok";

/// OAuth callback URL (deep link).
pub const OAUTH_REDIRECT_URL: &str = "openwok://auth/callback";
