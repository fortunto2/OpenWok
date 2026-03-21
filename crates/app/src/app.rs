#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::pages::home::Home;
use crate::pages::restaurants::{RestaurantList, RestaurantMenu};
use crate::state::{AppMode, CartState, PlatformConfig, UserState};

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
        #[route("/economics")]
        PublicEconomicsPage {},
        #[route("/operator")]
        OperatorConsole {},
}

#[component]
pub fn AppRoot() -> Element {
    use_context_provider(|| Signal::new(CartState::default()));
    use_context_provider(|| Signal::new(UserState::default()));
    use_context_provider(|| Signal::new(PlatformConfig::default()));
    use_context_provider(|| Signal::new(AppMode::default()));

    // Load config from server fn
    let mut config = use_context::<Signal<PlatformConfig>>();
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
    let user_state = use_context::<Signal<UserState>>();

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
                        span { class: "user-email",
                            "{user_state.read().email.as_deref().unwrap_or(\"User\")}"
                        }
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
    let mut mode = use_context::<Signal<AppMode>>();

    rsx! {
        nav { class: "bottom-tabs",
            match *mode.read() {
                AppMode::Customer => rsx! {
                    Link { to: Route::RestaurantList {}, class: "tab",
                        span { class: "tab-icon", "\u{1F35C}" }
                        span { class: "tab-label", "Food" }
                    }
                },
                AppMode::Courier => rsx! {
                    Link { to: Route::RestaurantList {}, class: "tab",
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
        }
    }
}

// Minimal page stubs for routes that are not fully migrated yet.
// These will be replaced with full implementations in future iterations.

#[component]
pub fn PublicEconomicsPage() -> Element {
    let economics = use_server_future(crate::server_fns::config::get_economics)?;

    match &*economics.read_unchecked() {
        Some(Ok(data)) => rsx! {
            h1 { "Open-Book Economics" }
            div { class: "economics-grid",
                div { class: "stat-card",
                    h3 { "Total Orders" }
                    p { "{data.total_orders}" }
                }
                div { class: "stat-card",
                    h3 { "Food Revenue" }
                    p { "${data.total_food_revenue}" }
                }
                div { class: "stat-card",
                    h3 { "Federal Fees ($1/order)" }
                    p { "${data.total_federal_fees}" }
                }
                div { class: "stat-card",
                    h3 { "Local Ops Fees" }
                    p { "${data.total_local_ops_fees}" }
                }
                div { class: "stat-card",
                    h3 { "Delivery Fees" }
                    p { "${data.total_delivery_fees}" }
                }
                div { class: "stat-card",
                    h3 { "Processing Fees" }
                    p { "${data.total_processing_fees}" }
                }
                div { class: "stat-card",
                    h3 { "Avg Order Value" }
                    p { "${data.avg_order_value}" }
                }
            }
        },
        Some(Err(e)) => rsx! { p { class: "error", "Error: {e}" } },
        None => rsx! { p { "Loading economics..." } },
    }
}

#[component]
pub fn OperatorConsole() -> Element {
    let metrics = use_server_future(crate::server_fns::config::get_admin_metrics)?;

    match &*metrics.read_unchecked() {
        Some(Ok(data)) => rsx! {
            h1 { "Operator Console" }
            div { class: "metrics-grid",
                div { class: "stat-card",
                    h3 { "Total Orders" }
                    p { "{data.order_count}" }
                }
                div { class: "stat-card",
                    h3 { "On-Time Rate" }
                    p { "{data.on_time_delivery_rate:.1}%" }
                }
                div { class: "stat-card",
                    h3 { "Avg ETA Error" }
                    p { "{data.avg_eta_error_minutes:.1} min" }
                }
                div { class: "stat-card",
                    h3 { "Couriers Available" }
                    p { "{data.courier_utilization.available}/{data.courier_utilization.total}" }
                }
            }
        },
        Some(Err(e)) => rsx! { p { class: "error", "Error: {e}" } },
        None => rsx! { p { "Loading metrics..." } },
    }
}
