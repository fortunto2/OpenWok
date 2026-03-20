//! # stripe-universal
//!
//! Stripe API client that works in native Rust (reqwest) and Cloudflare Workers (wasm32).
//!
//! ## Features
//!
//! - `reqwest-backend` (default) — native HTTP via reqwest
//! - `worker-backend` — Cloudflare Workers via worker::Fetch
//!
//! ## Usage
//!
//! ```rust,ignore
//! let stripe = StripeClient::new("sk_test_...");
//! let session = stripe.create_checkout_session(&params).await?;
//! // Redirect user to session.url
//! ```
//!
//! ## Webhook verification
//!
//! ```rust,ignore
//! let event = stripe_universal::webhook::verify_and_parse(
//!     body_bytes, signature_header, "whsec_...",
//! )?;
//! match event.event_type.as_str() {
//!     "checkout.session.completed" => { /* handle */ }
//!     _ => {}
//! }
//! ```

mod client;
mod error;
pub mod types;
pub mod webhook;

pub use client::StripeClient;
pub use error::StripeError;
