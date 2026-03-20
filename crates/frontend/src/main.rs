#![allow(non_snake_case)]

use dioxus::prelude::*;
use openwok_core::types::Restaurant;

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

const API_BASE: &str = "http://localhost:3000";

#[server]
async fn fetch_restaurants() -> Result<Vec<Restaurant>, ServerFnError> {
    let resp = reqwest::get(format!("{API_BASE}/restaurants")).await?;
    let data: Vec<Restaurant> = resp.json().await?;
    Ok(data)
}

#[component]
fn RestaurantList() -> Element {
    let restaurants = use_resource(fetch_restaurants);

    rsx! {
        h1 { "Restaurants" }
        match &*restaurants.read_unchecked() {
            Some(Ok(list)) if list.is_empty() => rsx! {
                p { "No restaurants available yet." }
            },
            Some(Ok(list)) => rsx! {
                div { class: "restaurant-grid",
                    for r in list {
                        RestaurantCard { key: "{r.id}", restaurant: r.clone() }
                    }
                }
            },
            Some(Err(e)) => rsx! {
                p { class: "error", "Failed to load restaurants: {e}" }
            },
            None => rsx! {
                p { "Loading restaurants..." }
            },
        }
    }
}

#[component]
fn RestaurantCard(restaurant: Restaurant) -> Element {
    rsx! {
        Link {
            to: Route::RestaurantMenu { id: restaurant.id.to_string() },
            class: "restaurant-card",
            h3 { "{restaurant.name}" }
            p { class: "item-count", "{restaurant.menu.len()} items" }
            div { class: "menu-preview",
                for item in restaurant.menu.iter().take(3) {
                    span { class: "menu-item", "{item.name} {item.price}" }
                }
            }
        }
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
