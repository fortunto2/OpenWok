#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::app::Route;
use crate::state::{UserState, save_jwt_to_storage};

#[cfg(target_arch = "wasm32")]
fn get_supabase_url() -> String {
    use wasm_bindgen::prelude::*;

    web_sys::window()
        .and_then(|window| {
            js_sys::Reflect::get(&window, &JsValue::from_str("__SUPABASE_URL__")).ok()
        })
        .and_then(|value| value.as_string())
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn get_supabase_url() -> String {
    String::new()
}

#[cfg(target_arch = "wasm32")]
fn get_origin() -> String {
    web_sys::window()
        .and_then(|window| window.location().origin().ok())
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn get_origin() -> String {
    String::new()
}

fn build_oauth_url() -> String {
    let supabase_url = get_supabase_url();
    if supabase_url.is_empty() {
        return String::new();
    }
    let origin = get_origin();
    format!("{supabase_url}/auth/v1/authorize?provider=google&redirect_to={origin}/auth/callback")
}

#[cfg(target_arch = "wasm32")]
fn get_callback_hash() -> String {
    web_sys::window()
        .and_then(|window| window.location().hash().ok())
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn get_callback_hash() -> String {
    String::new()
}

fn parse_access_token(hash: &str) -> Option<String> {
    hash.trim_start_matches('#').split('&').find_map(|part| {
        let (key, value) = part.split_once('=')?;
        if key == "access_token" {
            Some(value.to_string())
        } else {
            None
        }
    })
}

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

            match crate::server_fns::auth::auth_callback(token.clone()).await {
                Ok(user) => {
                    save_jwt_to_storage(&token);
                    user_state.set(UserState {
                        jwt: Some(token),
                        email: Some(user.email),
                    });
                    nav.push(Route::Home {});
                }
                Err(err) => {
                    error.set(Some(format!("Auth failed: {err}")));
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
