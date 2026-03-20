#![allow(non_snake_case)]

use dioxus::prelude::*;
use openwok_core::money::Money;
use openwok_core::pricing::calculate_pricing;
use rust_decimal::Decimal;

use crate::analytics::posthog_capture;
use crate::api::{cart_total, place_order};
use crate::app::Route;
use crate::state::CartState;

#[component]
pub fn Checkout() -> Element {
    let mut cart = use_context::<Signal<CartState>>();
    let mut address = use_signal(String::new);
    let mut tip_input = use_signal(|| "3.00".to_string());
    let mut order_error = use_signal(|| None::<String>);
    let mut placing = use_signal(|| false);
    let nav = use_navigator();

    let state = cart.read();

    if state.items.is_empty() {
        return rsx! {
            h1 { "Checkout" }
            p { "Your cart is empty. " }
            Link { to: Route::RestaurantList {}, "Browse restaurants" }
        };
    }

    posthog_capture("checkout_start");

    let food_total = cart_total(&state.items);
    let delivery_fee = Money::from("5.00");
    let local_ops_fee = Money::from("2.50");
    let tip = Money::from(tip_input.read().as_str());
    let pricing = calculate_pricing(food_total, delivery_fee, tip, local_ops_fee);
    let grand_total = pricing.total();

    let items_display: Vec<(String, u32, Money)> = state
        .items
        .iter()
        .map(|i| {
            let line = i.price * Decimal::from(i.quantity);
            (i.name.clone(), i.quantity, line)
        })
        .collect();

    let restaurant_name = state.restaurant_name.clone();
    let restaurant_id = state.restaurant_id.clone();
    let zone_id = state.zone_id.clone();

    let order_items: Vec<serde_json::Value> = state
        .items
        .iter()
        .map(|i| {
            serde_json::json!({
                "menu_item_id": i.menu_item_id,
                "name": i.name,
                "quantity": i.quantity,
                "unit_price": i.price.amount().to_string(),
            })
        })
        .collect();

    rsx! {
        div { class: "checkout-page",
            h1 { "Checkout" }
            h2 { "Order from {restaurant_name}" }

            // Cart summary
            div { class: "checkout-items",
                for (name, qty, line_total) in items_display {
                    div { class: "checkout-item",
                        span { "{name} x{qty}" }
                        span { "{line_total}" }
                    }
                }
            }

            // Delivery address
            div { class: "form-group",
                label { "Delivery Address" }
                input {
                    r#type: "text",
                    placeholder: "123 Main St, Los Angeles, CA",
                    value: "{address}",
                    oninput: move |e| address.set(e.value()),
                }
            }

            // Tip
            div { class: "form-group",
                label { "Tip ($)" }
                input {
                    r#type: "text",
                    value: "{tip_input}",
                    oninput: move |e| tip_input.set(e.value()),
                }
            }

            // 6-line pricing breakdown
            div { class: "pricing-breakdown",
                h3 { "Open-Book Receipt" }
                div { class: "price-line",
                    span { "Food Total" }
                    span { "{pricing.food_total}" }
                }
                div { class: "price-line",
                    span { "Delivery Fee" }
                    span { "{pricing.delivery_fee}" }
                }
                div { class: "price-line",
                    span { "Tip" }
                    span { "{pricing.tip}" }
                }
                div { class: "price-line",
                    span { "Federal Fee" }
                    span { "{pricing.federal_fee}" }
                }
                div { class: "price-line",
                    span { "Local Ops Fee" }
                    span { "{pricing.local_ops_fee}" }
                }
                div { class: "price-line",
                    span { "Processing (Stripe)" }
                    span { "{pricing.processing_fee}" }
                }
                div { class: "price-line total",
                    strong { "Total" }
                    strong { "{grand_total}" }
                }
            }

            // Error
            if let Some(err) = &*order_error.read() {
                p { class: "error", "{err}" }
            }

            // Place Order button
            button {
                class: "place-order-btn",
                disabled: placing() || address.read().is_empty(),
                onclick: move |_| {
                    let addr = address.read().clone();
                    let rid = restaurant_id.clone();
                    let zid = zone_id.clone();
                    let items_json = order_items.clone();
                    let tip_val = tip_input.read().clone();
                    async move {
                        placing.set(true);
                        let body = serde_json::json!({
                            "restaurant_id": rid,
                            "items": items_json,
                            "customer_address": addr,
                            "zone_id": zid,
                            "delivery_fee": "5.00",
                            "tip": tip_val,
                            "local_ops_fee": "2.50",
                        });
                        match place_order(body.to_string()).await {
                            Ok((order_id, checkout_url)) => {
                                posthog_capture("order_placed");
                                cart.write().items.clear();
                                if let Some(url) = checkout_url {
                                    // Redirect to Stripe Checkout
                                    if let Some(window) = web_sys::window() {
                                        let _ = window.location().set_href(&url);
                                    }
                                } else {
                                    // No Stripe configured — go to order tracking
                                    nav.push(Route::OrderTracking { id: order_id });
                                }
                            }
                            Err(e) => {
                                order_error.set(Some(e.to_string()));
                                placing.set(false);
                            }
                        }
                    }
                },
                if placing() { "Placing order..." } else { "Place Order" }
            }
        }
    }
}
