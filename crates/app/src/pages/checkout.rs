#![allow(non_snake_case)]

use dioxus::prelude::*;
use openwok_core::money::Money;
use openwok_core::pricing::calculate_pricing;
use rust_decimal::Decimal;

use crate::app::Route;
use crate::server_fns::orders::{CreateOrderInput, OrderItemInput};
use crate::state::{CartState, PlatformConfig};

#[component]
pub fn Checkout() -> Element {
    let mut cart = use_context::<Signal<CartState>>();
    let platform_config = use_context::<Signal<PlatformConfig>>();
    let mut address = use_signal(String::new);
    let config = platform_config.read();
    let default_tip = config.default_tip.clone();
    let mut tip_input = use_signal(move || default_tip.clone());
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

    let food_total = cart_total(&state.items);
    let delivery_fee = Money::from(config.delivery_fee.as_str());
    let local_ops_fee = Money::from(config.local_ops_fee.as_str());
    let tip = Money::from(tip_input.read().as_str());
    let pricing = calculate_pricing(food_total, delivery_fee, tip, local_ops_fee);
    let grand_total = pricing.total();

    let items_display: Vec<(String, u32, Money)> = state
        .items
        .iter()
        .map(|item| {
            let line = item.price * Decimal::from(item.quantity);
            (item.name.clone(), item.quantity, line)
        })
        .collect();

    let restaurant_name = state.restaurant_name.clone();
    let restaurant_id = state.restaurant_id.clone();
    let zone_id = state.zone_id.clone();

    let order_items: Vec<OrderItemInput> = state
        .items
        .iter()
        .map(|item| OrderItemInput {
            menu_item_id: parse_menu_item_id(&item.menu_item_id),
            name: item.name.clone(),
            quantity: item.quantity,
            unit_price: item.price,
        })
        .collect();

    rsx! {
        div { class: "checkout-page",
            h1 { "Checkout" }
            h2 { "Order from {restaurant_name}" }

            div { class: "checkout-items",
                for (name, qty, line_total) in items_display {
                    div { class: "checkout-item",
                        span { "{name} x{qty}" }
                        span { "{line_total}" }
                    }
                }
            }

            div { class: "form-group",
                label { "Delivery Address" }
                input {
                    r#type: "text",
                    placeholder: "123 Main St, Los Angeles, CA",
                    value: "{address}",
                    oninput: move |event| address.set(event.value()),
                }
            }

            div { class: "form-group",
                label { "Tip ($)" }
                input {
                    r#type: "text",
                    value: "{tip_input}",
                    oninput: move |event| tip_input.set(event.value()),
                }
            }

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

            if let Some(err) = &*order_error.read() {
                p { class: "error", "{err}" }
            }

            button {
                class: "place-order-btn",
                disabled: placing() || address.read().is_empty(),
                onclick: move |_| {
                    let addr = address.read().clone();
                    let rid = restaurant_id.clone();
                    let zid = zone_id.clone();
                    let items = order_items.clone();
                    let tip_val = tip_input.read().clone();
                    let cfg = platform_config.read().clone();
                    let nav = nav;
                    async move {
                        placing.set(true);
                        let input = CreateOrderInput {
                            restaurant_id: parse_restaurant_id(&rid),
                            items,
                            customer_address: addr,
                            zone_id: parse_zone_id(&zid),
                            delivery_fee: Money::from(cfg.delivery_fee.as_str()),
                            tip: Money::from(tip_val.as_str()),
                            local_ops_fee: Money::from(cfg.local_ops_fee.as_str()),
                        };
                        match crate::server_fns::orders::create_order(input).await {
                            Ok(order) => {
                                let mut cart_state = cart.write();
                                cart_state.items.clear();
                                cart_state.restaurant_id.clear();
                                cart_state.restaurant_name.clear();
                                cart_state.zone_id.clear();
                                nav.push(Route::OrderTracking {
                                    id: order.id.to_string(),
                                });
                            }
                            Err(error) => {
                                order_error.set(Some(error.to_string()));
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

fn cart_total(items: &[crate::state::CartItem]) -> Money {
    items
        .iter()
        .map(|item| item.price * Decimal::from(item.quantity))
        .fold(Money::zero(), |acc, line| acc + line)
}

fn parse_uuid(value: &str) -> uuid::Uuid {
    uuid::Uuid::parse_str(value).unwrap_or_else(|_| uuid::Uuid::nil())
}

fn parse_menu_item_id(value: &str) -> openwok_core::types::MenuItemId {
    openwok_core::types::MenuItemId::from_uuid(parse_uuid(value))
}

fn parse_restaurant_id(value: &str) -> openwok_core::types::RestaurantId {
    openwok_core::types::RestaurantId::from_uuid(parse_uuid(value))
}

fn parse_zone_id(value: &str) -> openwok_core::types::ZoneId {
    openwok_core::types::ZoneId::from_uuid(parse_uuid(value))
}
