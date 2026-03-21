#![allow(non_snake_case)]

use dioxus::prelude::*;

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
        #[route("/economics")]
        PublicEconomicsPage {},
        #[route("/operator")]
        OperatorConsole {},
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
pub fn AppRoot() -> Element {
    use_context_provider(|| Signal::new(CartState::default()));
    use_context_provider(|| {
        let jwt = get_jwt_from_storage();
        Signal::new(UserState { jwt, email: None })
    });
    use_context_provider(|| Signal::new(PlatformConfig::default()));
    use_context_provider(|| Signal::new(AppMode::default()));

    // Load config from server fn
    let mut config = use_context::<Signal<PlatformConfig>>();
    let mut user_state = use_context::<Signal<UserState>>();
    use_effect(move || {
        spawn(async move {
            if let Ok(data) = crate::server_fns::config::get_config().await {
                config.set(PlatformConfig {
                    delivery_fee: data.default_delivery_fee,
                    local_ops_fee: data.default_local_ops_fee,
                    federal_fee: data.federal_fee,
                    default_tip: "3.00".into(),
                });
            }

            let jwt = user_state.read().jwt.clone();
            if let Some(jwt) = jwt {
                match crate::server_fns::auth::get_me(jwt).await {
                    Ok(user) => {
                        user_state.write().email = Some(user.email);
                    }
                    Err(_) => {
                        clear_jwt_from_storage();
                        user_state.set(UserState::default());
                    }
                }
            }
        });
    });

    rsx! {
        ErrorBoundary {
            handle_error: |errors: ErrorContext| {
                rsx! {
                    div { class: "error-boundary",
                        h1 { "Something went wrong" }
                        p { "{errors:?}" }
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
        MobileTabBar {}
    }
}

#[component]
fn MobileTabBar() -> Element {
    let user_state = use_context::<Signal<UserState>>();
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
            if user_state.read().jwt.is_none() {
                Link { to: Route::Login {}, class: "tab",
                    span { class: "tab-icon", "\u{1F511}" }
                    span { class: "tab-label", "Login" }
                }
            }
        }
    }
}
