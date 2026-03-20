#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::api::api_post_json;
use crate::app::Route;
use crate::state::{UserState, save_jwt_to_storage};

// --- Supabase OAuth URL helpers ---

#[cfg(target_arch = "wasm32")]
fn get_supabase_url() -> String {
    use wasm_bindgen::prelude::*;
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("__SUPABASE_URL__")).ok())
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn get_supabase_url() -> String {
    // AI-TODO: load from config file or env
    String::new()
}

#[cfg(target_arch = "wasm32")]
fn get_origin() -> String {
    web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn get_origin() -> String {
    "openwok://".to_string()
}

fn build_oauth_url() -> String {
    let supabase_url = get_supabase_url();
    if supabase_url.is_empty() {
        return String::new();
    }
    let origin = get_origin();
    format!("{supabase_url}/auth/v1/authorize?provider=google&redirect_to={origin}/auth/callback")
}

// --- Extract access_token from callback ---

#[cfg(target_arch = "wasm32")]
fn get_callback_hash() -> String {
    web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn get_callback_hash() -> String {
    // AI-TODO: deep link callback handling
    String::new()
}

fn parse_access_token(hash: &str) -> Option<String> {
    hash.trim_start_matches('#').split('&').find_map(|part| {
        let (key, val) = part.split_once('=')?;
        if key == "access_token" {
            Some(val.to_string())
        } else {
            None
        }
    })
}

// --- Components ---

#[component]
pub fn Login() -> Element {
    let user_state = use_context::<Signal<UserState>>();

    if user_state.read().jwt.is_some() {
        return rsx! {
            p { "Already signed in. " }
            Link { to: Route::Home {}, "Go home" }
        };
    }

    let oauth_url = build_oauth_url();

    rsx! {
        div { class: "login-page",
            h1 { "Sign In" }
            p { "Sign in to place orders and track deliveries." }
            if oauth_url.is_empty() {
                p { class: "error", "Supabase not configured." }
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

    use_effect(move || {
        spawn(async move {
            let hash = get_callback_hash();
            let Some(token) = parse_access_token(&hash) else {
                error.set(Some("No access_token in callback URL".into()));
                return;
            };

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
