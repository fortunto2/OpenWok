#![allow(non_snake_case)]

use dioxus::prelude::*;
use wasm_bindgen::prelude::*;

use crate::api::api_post_json;
use crate::app::Route;
use crate::state::{UserState, save_jwt_to_storage};

#[component]
pub fn Login() -> Element {
    let user_state = use_context::<Signal<UserState>>();

    // Already logged in → redirect home
    if user_state.read().jwt.is_some() {
        return rsx! {
            p { "Already signed in. " }
            Link { to: Route::Home {}, "Go home" }
        };
    }

    // Build Supabase OAuth URL from window.__SUPABASE_URL__ or default
    let supabase_url = web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("__SUPABASE_URL__")).ok())
        .and_then(|v| v.as_string())
        .unwrap_or_default();

    let origin = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default();

    let oauth_url = if supabase_url.is_empty() {
        String::new()
    } else {
        format!(
            "{supabase_url}/auth/v1/authorize?provider=google&redirect_to={origin}/auth/callback"
        )
    };

    rsx! {
        div { class: "login-page",
            h1 { "Sign In" }
            p { "Sign in to place orders and track deliveries." }
            if oauth_url.is_empty() {
                p { class: "error", "Supabase not configured. Set window.__SUPABASE_URL__ to enable Google OAuth." }
            } else {
                a {
                    href: "{oauth_url}",
                    class: "google-signin-btn cta",
                    "Sign in with Google"
                }
            }
        }
    }
}

#[component]
pub fn AuthCallback() -> Element {
    let mut user_state = use_context::<Signal<UserState>>();
    let nav = use_navigator();
    let mut error = use_signal(|| None::<String>);

    // Extract access_token from URL hash fragment on mount
    use_effect(move || {
        spawn(async move {
            // Get hash fragment: #access_token=xxx&token_type=bearer&...
            let hash = web_sys::window()
                .and_then(|w| w.location().hash().ok())
                .unwrap_or_default();

            let access_token = hash.trim_start_matches('#').split('&').find_map(|part| {
                let (key, val) = part.split_once('=')?;
                if key == "access_token" {
                    Some(val.to_string())
                } else {
                    None
                }
            });

            let Some(token) = access_token else {
                error.set(Some("No access_token in callback URL".into()));
                return;
            };

            // Call backend to verify token and create/get user
            let body = serde_json::json!({ "access_token": token });
            let resp: Result<serde_json::Value, String> =
                api_post_json("/auth/callback", &body.to_string()).await;

            match resp {
                Ok(data) => {
                    let email = data["user"]["email"].as_str().map(|s| s.to_string());
                    save_jwt_to_storage(&token);
                    user_state.set(UserState {
                        jwt: Some(token),
                        email,
                    });
                    nav.push(Route::Home {});
                }
                Err(e) => {
                    error.set(Some(format!("Auth failed: {e}")));
                }
            }
        });
    });

    rsx! {
        div { class: "auth-callback",
            if let Some(err) = &*error.read() {
                p { class: "error", "{err}" }
                Link { to: Route::Login {}, "Try again" }
            } else {
                p { "Signing you in..." }
            }
        }
    }
}
