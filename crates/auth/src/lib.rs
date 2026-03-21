//! # superduperai-auth
//!
//! Shared authentication for SuperDuperAi apps.
//! Supabase Auth (GoTrue) + Google OAuth + JWT verification + axum middleware.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use superduperai_auth::{AuthClient, AuthConfig};
//!
//! let config = AuthConfig::new("https://xxx.supabase.co", "anon-key", "openwok");
//! let auth = AuthClient::new(config);
//!
//! // Sign in with Google ID token
//! let session = auth.sign_in_with_id_token("google-id-token").await?;
//!
//! // Sign in with email+password
//! let session = auth.sign_in_with_password("user@email.com", "password").await?;
//!
//! // Verify JWT (for server-side middleware)
//! let claims = auth.verify_token(&session.access_token)?;
//! ```

mod client;
mod config;
mod error;
mod types;

#[cfg(feature = "axum-middleware")]
pub mod middleware;

pub use client::AuthClient;
pub use config::AuthConfig;
pub use error::{AuthError, AuthErrorCode};
pub use types::{Claims, Session, User, UserProfile};
