#![allow(non_snake_case)]

use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq, Routable)]
#[rustfmt::skip]
enum Route {
    #[layout(Layout)]
        #[route("/")]
        Home {},
        #[route("/restaurants")]
        RestaurantList {},
        #[route("/restaurant/:id")]
        RestaurantMenu { id: String },
        #[route("/order/:id")]
        OrderTracking { id: String },
        #[route("/operator")]
        OperatorConsole {},
}

#[component]
fn App() -> Element {
    rsx! { Router::<Route> {} }
}

#[component]
fn Layout() -> Element {
    rsx! {
        header { class: "header",
            nav { class: "nav",
                Link { to: Route::Home {}, class: "logo", "🍜 OpenWok" }
                div { class: "nav-links",
                    Link { to: Route::RestaurantList {}, "Restaurants" }
                    Link { to: Route::OperatorConsole {}, "Operator" }
                }
            }
        }
        main { class: "content",
            Outlet::<Route> {}
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        div { class: "hero",
            h1 { "OpenWok" }
            p { class: "subtitle", "Fair food delivery. $1 fee. Open-book pricing." }
            Link { to: Route::RestaurantList {}, class: "cta", "Browse Restaurants" }
        }
    }
}

#[component]
fn RestaurantList() -> Element {
    rsx! {
        h1 { "Restaurants" }
        p { "Loading restaurants..." }
    }
}

#[component]
fn RestaurantMenu(id: String) -> Element {
    rsx! {
        h1 { "Restaurant Menu" }
        p { "Restaurant: {id}" }
    }
}

#[component]
fn OrderTracking(id: String) -> Element {
    rsx! {
        h1 { "Order Tracking" }
        p { "Order: {id}" }
    }
}

#[component]
fn OperatorConsole() -> Element {
    rsx! {
        h1 { "Node Operator Console" }
        p { "Dashboard coming soon..." }
    }
}

fn main() {
    dioxus::launch(App);
}
