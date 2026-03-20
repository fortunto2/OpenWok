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
use crate::state::{CartState, UserState, clear_jwt_from_storage, get_jwt_from_storage};

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
    rsx! {
        document::Script { {POSTHOG_SNIPPET} }
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
    }
}
