#![allow(non_snake_case)]

use dioxus::prelude::*;
use openwok_core::money::Money;
use openwok_core::types::Restaurant;
use rust_decimal::Decimal;

#[derive(Clone, PartialEq)]
struct CartItem {
    name: String,
    price: Money,
    quantity: u32,
}

#[derive(Clone, Default, PartialEq)]
struct CartState {
    items: Vec<CartItem>,
    restaurant_name: String,
}

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
    use_context_provider(|| Signal::new(CartState::default()));
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

#[server]
async fn fetch_restaurant(id: String) -> Result<Restaurant, ServerFnError> {
    let resp = reqwest::get(format!("{API_BASE}/restaurants/{id}")).await?;
    if !resp.status().is_success() {
        return Err(ServerFnError::ServerError("Restaurant not found".into()));
    }
    Ok(resp.json().await?)
}

fn cart_total(items: &[CartItem]) -> Money {
    items
        .iter()
        .map(|i| i.price * Decimal::from(i.quantity))
        .fold(Money::zero(), |a, b| a + b)
}

#[component]
fn RestaurantMenu(id: String) -> Element {
    let restaurant = use_resource(move || {
        let id = id.clone();
        async move { fetch_restaurant(id).await }
    });
    let mut cart = use_context::<Signal<CartState>>();

    match &*restaurant.read_unchecked() {
        Some(Ok(r)) => {
            let r = r.clone();
            rsx! {
                div { class: "menu-page",
                    div { class: "menu-section",
                        h1 { "{r.name}" }
                        div { class: "menu-items",
                            for item in r.menu.iter() {
                                div { class: "menu-item-row",
                                    div { class: "menu-item-info",
                                        span { class: "menu-item-name", "{item.name}" }
                                        span { class: "menu-item-price", "{item.price}" }
                                    }
                                    button {
                                        class: "add-btn",
                                        onclick: {
                                            let name = item.name.clone();
                                            let price = item.price;
                                            let restaurant_name = r.name.clone();
                                            move |_| {
                                                let mut state = cart.write();
                                                state.restaurant_name = restaurant_name.clone();
                                                if let Some(existing) = state.items.iter_mut().find(|c| c.name == name) {
                                                    existing.quantity += 1;
                                                } else {
                                                    state.items.push(CartItem {
                                                        name: name.clone(),
                                                        price,
                                                        quantity: 1,
                                                    });
                                                }
                                            }
                                        },
                                        "Add"
                                    }
                                }
                            }
                        }
                    }
                    CartPanel {}
                }
            }
        }
        Some(Err(e)) => rsx! { p { class: "error", "Error: {e}" } },
        None => rsx! { p { "Loading menu..." } },
    }
}

#[component]
fn CartPanel() -> Element {
    let cart = use_context::<Signal<CartState>>();
    let state = cart.read();

    if state.items.is_empty() {
        return rsx! {
            div { class: "cart-panel",
                h2 { "Cart" }
                p { "Your cart is empty" }
            }
        };
    }

    let total = cart_total(&state.items);
    let items: Vec<(String, u32, Money)> = state
        .items
        .iter()
        .map(|i| {
            let line = i.price * Decimal::from(i.quantity);
            (i.name.clone(), i.quantity, line)
        })
        .collect();

    rsx! {
        div { class: "cart-panel",
            h2 { "Cart" }
            for (name, qty, line_total) in items {
                div { class: "cart-item",
                    span { "{name} x{qty}" }
                    span { "{line_total}" }
                }
            }
            div { class: "cart-total",
                strong { "Total: {total}" }
            }
            Link { to: Route::Home {}, class: "checkout-btn", "Proceed to Order" }
        }
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
