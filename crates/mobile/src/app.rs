#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::pages::checkout::Checkout;
use crate::pages::courier::{MyDeliveries, RegisterCourier};
use crate::pages::order::{OrderSuccess, OrderTracking};
use crate::pages::restaurants::{RestaurantList, RestaurantMenu};
use crate::state::{AppMode, CartState, UserState};
use crate::storage;

// --- Routes ---

#[derive(Clone, Debug, PartialEq, Routable)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Layout)]
        #[route("/")]
        RestaurantList {},
        #[route("/restaurant/:id")]
        RestaurantMenu { id: String },
        #[route("/checkout")]
        Checkout {},
        #[route("/order/:id")]
        OrderTracking { id: String },
        #[route("/order/:id/success")]
        OrderSuccess { id: String },
        #[route("/login")]
        Login {},
        #[route("/auth/callback")]
        AuthCallback {},
        #[route("/register-courier")]
        RegisterCourier {},
        #[route("/my-deliveries")]
        MyDeliveries {},
}

// --- Login page ---

#[component]
fn Login() -> Element {
    let user_state = use_context::<Signal<UserState>>();

    if user_state.read().jwt.is_some() {
        return rsx! {
            div { class: "page login-page",
                p { "Already signed in." }
                Link { to: Route::RestaurantList {}, class: "cta", "Browse Restaurants" }
            }
        };
    }

    rsx! {
        div { class: "page login-page",
            div { class: "login-content",
                h1 { "OpenWok" }
                p { "Fair food delivery" }
                button {
                    class: "cta google-btn",
                    onclick: move |_| {
                        crate::auth::start_oauth();
                    },
                    "Sign in with Google"
                }
            }
        }
    }
}

// --- Auth callback ---

#[component]
fn AuthCallback() -> Element {
    // Deep link callback handling will be wired when deep links are available.
    // For now, show a placeholder that explains the flow.
    rsx! {
        div { class: "page auth-callback",
            p { "Signing you in..." }
        }
    }
}

// --- Layout with bottom tab bar ---

#[component]
fn Layout() -> Element {
    let mut user_state = use_context::<Signal<UserState>>();
    let mut mode = use_context::<Signal<AppMode>>();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/style.css") }
        main { class: "content",
            Outlet::<Route> {}
        }
        nav { class: "bottom-tabs",
            match *mode.read() {
                AppMode::Customer => rsx! {
                    Link { to: Route::RestaurantList {}, class: "tab",
                        span { class: "tab-icon", "🍜" }
                        span { class: "tab-label", "Restaurants" }
                    }
                    Link { to: Route::Checkout {}, class: "tab",
                        span { class: "tab-icon", "🛒" }
                        span { class: "tab-label", "Cart" }
                    }
                },
                AppMode::Courier => rsx! {
                    Link { to: Route::MyDeliveries {}, class: "tab",
                        span { class: "tab-icon", "📦" }
                        span { class: "tab-label", "Deliveries" }
                    }
                },
            }
            // Mode switcher tab
            {
                let current_mode = mode.read().clone();
                rsx! {
                    button {
                        class: "tab mode-switch",
                        onclick: move |_| {
                            let mut m = mode.write();
                            *m = match *m {
                                AppMode::Customer => AppMode::Courier,
                                AppMode::Courier => AppMode::Customer,
                            };
                        },
                        span { class: "tab-icon",
                            match current_mode {
                                AppMode::Customer => "🚴",
                                AppMode::Courier => "🍽️",
                            }
                        }
                        span { class: "tab-label",
                            match current_mode {
                                AppMode::Customer => "Deliver",
                                AppMode::Courier => "Order",
                            }
                        }
                    }
                }
            }
            // Profile tab
            if user_state.read().jwt.is_some() {
                button {
                    class: "tab",
                    onclick: move |_| {
                        crate::auth::logout();
                        user_state.write().jwt = None;
                        user_state.write().email = None;
                    },
                    span { class: "tab-icon", "👤" }
                    span { class: "tab-label", "Logout" }
                }
            } else {
                Link { to: Route::Login {}, class: "tab",
                    span { class: "tab-icon", "👤" }
                    span { class: "tab-label", "Sign In" }
                }
            }
        }
    }
}

// --- Root App ---

pub fn App() -> Element {
    use_context_provider(|| Signal::new(CartState::default()));
    use_context_provider(|| {
        let jwt = storage::load_jwt();
        Signal::new(UserState { jwt, email: None })
    });
    use_context_provider(|| Signal::new(AppMode::default()));

    rsx! {
        Router::<Route> {}
    }
}
