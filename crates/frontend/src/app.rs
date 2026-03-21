#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::analytics::{POSTHOG_SNIPPET, posthog_capture_with_props};
use crate::pages::auth::{AuthCallback, Login};
use crate::pages::checkout::Checkout;
use crate::pages::courier::{MyDeliveries, RegisterCourier};
use crate::pages::economics::PublicEconomicsPage;
use crate::pages::home::Home;
use crate::pages::operator::OperatorConsole;
use crate::pages::order::{OrderSuccess, OrderTracking};
use crate::pages::owner::{MyRestaurants, OnboardRestaurant, RestaurantSettings};
use crate::pages::restaurants::{RestaurantList, RestaurantMenu};
use crate::state::{
    AppMode, CartState, PlatformConfig, UserState, clear_jwt_from_storage, get_jwt_from_storage,
};

#[cfg(target_arch = "wasm32")]
const SW_REGISTER: &str = r#"
if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('/sw.js').catch(function(e) {
        console.warn('SW registration failed:', e);
    });
}
"#;

#[cfg(not(target_arch = "wasm32"))]
const SW_REGISTER: &str = "";

#[derive(Clone, Debug, PartialEq, Routable)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Layout)]
        #[route("/")]
        Home {},
        #[route("/restaurants")]
        RestaurantList {},
        #[route("/restaurant/:id")]
        RestaurantMenu { id: String },
        #[route("/checkout")]
        Checkout {},
        #[route("/order/:id")]
        OrderTracking { id: String },
        #[route("/order/:id/success")]
        OrderSuccess { id: String },
        #[route("/operator")]
        OperatorConsole {},
        #[route("/economics")]
        PublicEconomicsPage {},
        #[route("/login")]
        Login {},
        #[route("/auth/callback")]
        AuthCallback {},
        #[route("/my-restaurants")]
        MyRestaurants {},
        #[route("/my-restaurants/:id")]
        RestaurantSettings { id: String },
        #[route("/onboard-restaurant")]
        OnboardRestaurant {},
        #[route("/register-courier")]
        RegisterCourier {},
        #[route("/my-deliveries")]
        MyDeliveries {},
}

#[component]
pub fn App() -> Element {
    use_context_provider(|| Signal::new(CartState::default()));
    use_context_provider(|| {
        let jwt = get_jwt_from_storage();
        Signal::new(UserState { jwt, email: None })
    });
    use_context_provider(|| Signal::new(PlatformConfig::default()));
    use_context_provider(|| Signal::new(AppMode::default()));
    use_context_provider(crate::local_db::create_store);

    // Fetch config from API on startup
    let mut config = use_context::<Signal<PlatformConfig>>();
    use_effect(move || {
        spawn(async move {
            if let Ok(data) = crate::api::fetch_config().await {
                config.set(PlatformConfig {
                    delivery_fee: data["delivery_fee"].as_str().unwrap_or("5.00").into(),
                    local_ops_fee: data["local_ops_fee"].as_str().unwrap_or("2.50").into(),
                    federal_fee: data["federal_fee"].as_str().unwrap_or("1.00").into(),
                    default_tip: data["default_tip"].as_str().unwrap_or("3.00").into(),
                });
            }
        });
    });

    // Background sync loop: pull deliveries + push pending every 15s
    let store = use_context::<crate::local_db::Store>();
    use_future(move || {
        let store = store.clone();
        async move {
            loop {
                if crate::platform::is_online() {
                    crate::sync::push_pending(store.as_ref()).await;
                    crate::sync::pull_deliveries(store.as_ref()).await;
                }
                crate::platform::sleep_ms(15_000).await;
            }
        }
    });

    rsx! {
        document::Script { {POSTHOG_SNIPPET} }
        document::Script { {SW_REGISTER} }
        ErrorBoundary {
            handle_error: |errors: ErrorContext| {
                let error_text = format!("{errors:?}");
                posthog_capture_with_props(
                    "frontend_error",
                    &serde_json::json!({ "error": error_text }),
                );
                rsx! {
                    div { class: "error-boundary",
                        h1 { "Something went wrong" }
                        p { "We're sorry — an unexpected error occurred. Please try refreshing the page." }
                        button {
                            class: "cta",
                            onclick: move |_| {
                                crate::platform::reload_page();
                            },
                            "Reload Page"
                        }
                    }
                }
            },
            Router::<Route> {}
        }
    }
}

#[component]
fn Layout() -> Element {
    let mut user_state = use_context::<Signal<UserState>>();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/tailwind.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/style.css") }
        header { class: "header",
            nav { class: "nav",
                Link { to: Route::Home {}, class: "logo", "OpenWok" }
                div { class: "nav-links",
                    Link { to: Route::RestaurantList {}, "Restaurants" }
                    Link { to: Route::PublicEconomicsPage {}, "Economics" }
                    Link { to: Route::OperatorConsole {}, "Operator" }
                    if user_state.read().jwt.is_some() {
                        Link { to: Route::MyRestaurants {}, "My Restaurants" }
                        Link { to: Route::MyDeliveries {}, "My Deliveries" }
                        span { class: "user-email",
                            "{user_state.read().email.as_deref().unwrap_or(\"User\")}"
                        }
                        button {
                            class: "logout-btn",
                            onclick: move |_| {
                                clear_jwt_from_storage();
                                user_state.set(UserState::default());
                            },
                            "Logout"
                        }
                    } else {
                        Link { to: Route::Login {}, class: "login-link", "Sign In" }
                    }
                }
            }
        }
        main { class: "content",
            Outlet::<Route> {}
        }
        // Mobile bottom tab bar (hidden on desktop via CSS)
        MobileTabBar {}
    }
}

#[component]
fn MobileTabBar() -> Element {
    let mut user_state = use_context::<Signal<UserState>>();
    let mut mode = use_context::<Signal<AppMode>>();

    rsx! {
        nav { class: "bottom-tabs",
            match *mode.read() {
                AppMode::Customer => rsx! {
                    Link { to: Route::RestaurantList {}, class: "tab",
                        span { class: "tab-icon", "\u{1F35C}" }
                        span { class: "tab-label", "Food" }
                    }
                    Link { to: Route::Checkout {}, class: "tab",
                        span { class: "tab-icon", "\u{1F6D2}" }
                        span { class: "tab-label", "Cart" }
                    }
                },
                AppMode::Courier => rsx! {
                    Link { to: Route::MyDeliveries {}, class: "tab",
                        span { class: "tab-icon", "\u{1F4E6}" }
                        span { class: "tab-label", "Deliveries" }
                    }
                },
            }
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
                    match *mode.read() {
                        AppMode::Customer => "\u{1F6B4}",
                        AppMode::Courier => "\u{1F37D}\u{FE0F}",
                    }
                }
                span { class: "tab-label",
                    match *mode.read() {
                        AppMode::Customer => "Deliver",
                        AppMode::Courier => "Order",
                    }
                }
            }
            if user_state.read().jwt.is_some() {
                button {
                    class: "tab",
                    onclick: move |_| {
                        clear_jwt_from_storage();
                        user_state.set(UserState::default());
                    },
                    span { class: "tab-icon", "\u{1F464}" }
                    span { class: "tab-label", "Logout" }
                }
            } else {
                Link { to: Route::Login {}, class: "tab",
                    span { class: "tab-icon", "\u{1F464}" }
                    span { class: "tab-label", "Sign In" }
                }
            }
        }
    }
}
