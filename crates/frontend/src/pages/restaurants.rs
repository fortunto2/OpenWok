#![allow(non_snake_case)]

use dioxus::prelude::*;
use openwok_core::types::Restaurant;
use rust_decimal::Decimal;

use crate::analytics::posthog_capture;
use crate::api::{cart_total, fetch_restaurant, fetch_restaurants};
use crate::app::Route;
use crate::state::{CartItem, CartState};

#[component]
pub fn RestaurantList() -> Element {
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
pub fn RestaurantMenu(id: String) -> Element {
    let restaurant = use_resource(move || {
        let id = id.clone();
        async move { fetch_restaurant(id).await }
    });
    let mut cart = use_context::<Signal<CartState>>();

    match &*restaurant.read_unchecked() {
        Some(Ok(r)) => {
            posthog_capture("restaurant_view");
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
                                            let menu_item_id = item.id.to_string();
                                            let name = item.name.clone();
                                            let price = item.price;
                                            let rest_id = r.id.to_string();
                                            let rest_name = r.name.clone();
                                            let zone = r.zone_id.to_string();
                                            move |_| {
                                                posthog_capture("add_to_cart");
                                                let mut state = cart.write();
                                                state.restaurant_id = rest_id.clone();
                                                state.restaurant_name = rest_name.clone();
                                                state.zone_id = zone.clone();
                                                if let Some(existing) = state.items.iter_mut().find(|c| c.menu_item_id == menu_item_id) {
                                                    existing.quantity += 1;
                                                } else {
                                                    state.items.push(CartItem {
                                                        menu_item_id: menu_item_id.clone(),
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
    let items: Vec<(String, u32, openwok_core::money::Money)> = state
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
            Link { to: Route::Checkout {}, class: "checkout-btn", "Proceed to Order" }
        }
    }
}
