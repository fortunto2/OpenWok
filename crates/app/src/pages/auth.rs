#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::app::Route;
use crate::state::{UserState, save_jwt_to_storage};

#[derive(Clone, Default)]
struct LoginConfig {
    app_url: String,
    supabase_url: String,
    google_oauth_enabled: bool,
}

#[derive(Clone, Default)]
struct PasswordAuthForm {
    email: String,
    password: String,
}

fn initial_login_config() -> LoginConfig {
    LoginConfig::default()
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

fn build_oauth_url(config: &LoginConfig) -> String {
    let supabase_url = config.supabase_url.as_str();
    if supabase_url.is_empty() {
        return String::new();
    }
    let origin = if config.app_url.is_empty() {
        get_origin()
    } else {
        config.app_url.clone()
    };
    if origin.is_empty() {
        return String::new();
    }
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
    let mut user_state = use_context::<Signal<UserState>>();
    let nav = use_navigator();
    let mut login_config = use_signal(initial_login_config);
    let mut settings_loaded = use_signal(|| false);
    let mut form = use_signal(PasswordAuthForm::default);
    let mut status = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            if let Ok(settings) = crate::server_fns::auth::get_auth_settings().await {
                login_config.set(LoginConfig {
                    app_url: settings.app_url,
                    supabase_url: settings.supabase_url,
                    google_oauth_enabled: settings.google_oauth_enabled,
                });
            }
            settings_loaded.set(true);
        });
    });

    if user_state.read().jwt.is_some() {
        return rsx! {
            p { "Already signed in. " }
            Link { to: Route::Home {}, "Go home" }
        };
    }

    let oauth_url = build_oauth_url(&login_config.read());
    let is_configured = !login_config.read().supabase_url.is_empty();

    rsx! {
        div { class: "login-page",
            h1 { "Sign In" }
            p { "Sign in to place orders and track deliveries." }
            if !settings_loaded() && !is_configured {
                p { "Loading sign-in options..." }
            } else if !is_configured {
                p { class: "error", "Supabase not configured." }
            } else {
                div { class: "form-group",
                    label { r#for: "email-login", "Email" }
                    input {
                        id: "email-login",
                        r#type: "email",
                        autocomplete: "email",
                        value: "{form.read().email}",
                        oninput: move |evt| {
                            let mut next = form.read().clone();
                            next.email = evt.value();
                            form.set(next);
                        }
                    }
                }
                div { class: "form-group",
                    label { r#for: "password-login", "Password" }
                    input {
                        id: "password-login",
                        r#type: "password",
                        autocomplete: "current-password",
                        value: "{form.read().password}",
                        oninput: move |evt| {
                            let mut next = form.read().clone();
                            next.password = evt.value();
                            form.set(next);
                        }
                    }
                }
                div { class: "auth-actions",
                    button {
                        class: "cta",
                        disabled: loading(),
                        onclick: move |_| {
                            let credentials = form.read().clone();
                            if credentials.email.trim().is_empty() || credentials.password.is_empty() {
                                error.set(Some("Enter both email and password.".into()));
                                status.set(None);
                                return;
                            }
                            loading.set(true);
                            error.set(None);
                            status.set(None);
                            spawn(async move {
                                match crate::server_fns::auth::sign_in_with_email_password(
                                    credentials.email.clone(),
                                    credentials.password.clone(),
                                )
                                .await
                                {
                                    Ok(result) => {
                                        if let Some(message) = result.error {
                                            error.set(Some(message.message));
                                        } else if let (Some(token), Some(user)) =
                                            (result.token, result.user)
                                        {
                                            save_jwt_to_storage(&token);
                                            user_state.set(UserState {
                                                jwt: Some(token),
                                                email: Some(user.email),
                                            });
                                            nav.push(Route::Home {});
                                        } else {
                                            error.set(Some("Sign-in did not return a session.".into()));
                                        }
                                    }
                                    Err(err) => {
                                        error.set(Some(err.to_string()));
                                    }
                                }
                                loading.set(false);
                            });
                        },
                        if loading() { "Signing in..." } else { "Sign in with Email" }
                    }
                    button {
                        class: "secondary-btn",
                        disabled: loading(),
                        onclick: move |_| {
                            let credentials = form.read().clone();
                            if credentials.email.trim().is_empty() || credentials.password.len() < 8 {
                                error.set(Some("Use a valid email and a password of at least 8 characters.".into()));
                                status.set(None);
                                return;
                            }
                            loading.set(true);
                            error.set(None);
                            status.set(None);
                            spawn(async move {
                                match crate::server_fns::auth::sign_up_with_email_password(
                                    credentials.email.clone(),
                                    credentials.password.clone(),
                                )
                                .await
                                {
                                    Ok(result) => {
                                        if let Some(message) = result.error {
                                            error.set(Some(message.message));
                                        } else {
                                            let email = result.email.unwrap_or_else(|| credentials.email.clone());
                                            let message = if result.confirmation_required {
                                                format!("Check {} for a confirmation email.", email)
                                            } else {
                                                format!("Account created for {}. You can sign in now.", email)
                                            };
                                            status.set(Some(message));
                                        }
                                    }
                                    Err(err) => {
                                        error.set(Some(err.to_string()));
                                    }
                                }
                                loading.set(false);
                            });
                        },
                        if loading() { "Working..." } else { "Create Account" }
                    }
                }
                if let Some(message) = &*status.read() {
                    p { class: "success", "{message}" }
                }
                if let Some(message) = &*error.read() {
                    p { class: "error", "{message}" }
                }
                if login_config.read().google_oauth_enabled && !oauth_url.is_empty() {
                    p { class: "auth-separator", "or" }
                    a {
                        href: "{oauth_url}",
                        class: "google-signin-btn cta",
                        "Sign in with Google"
                    }
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
