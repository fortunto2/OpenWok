#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::app::Route;
use crate::local_db::Store;

const ORDER_TIMELINE: &[&str] = &[
    "Created",
    "Confirmed",
    "Preparing",
    "ReadyForPickup",
    "InDelivery",
    "Delivered",
];

#[component]
pub fn OrderSuccess(id: String) -> Element {
    rsx! {
        div { class: "order-success",
            h1 { "Payment Successful!" }
            p { "Your order has been confirmed. You can track it below." }
            Link {
                to: Route::OrderTracking { id: id.clone() },
                class: "cta",
                "Track Order"
            }
        }
    }
}

#[component]
pub fn OrderTracking(id: String) -> Element {
    let store = use_context::<Store>();
    let mut refresh = use_signal(|| 0u32);
    let order = use_resource(move || {
        let _ = refresh();
        let id = id.clone();
        let store = store.clone();
        async move {
            crate::api::cached_get::<serde_json::Value>(
                &format!("/orders/{id}"),
                store.as_ref(),
                &format!("order_{id}"),
            )
            .await
        }
    });

    match &*order.read_unchecked() {
        Some(Ok(data)) => {
            let status = data["status"].as_str().unwrap_or("Unknown").to_string();
            let pricing = &data["pricing"];

            let food_total = pricing["food_total"].as_str().unwrap_or("0").to_string();
            let delivery_fee = pricing["delivery_fee"].as_str().unwrap_or("0").to_string();
            let tip = pricing["tip"].as_str().unwrap_or("0").to_string();
            let federal_fee = pricing["federal_fee"].as_str().unwrap_or("0").to_string();
            let local_ops_fee = pricing["local_ops_fee"].as_str().unwrap_or("0").to_string();
            let processing_fee = pricing["processing_fee"]
                .as_str()
                .unwrap_or("0")
                .to_string();

            let items: Vec<(String, u64, String)> = data["items"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|i| {
                            (
                                i["name"].as_str().unwrap_or("").to_string(),
                                i["quantity"].as_u64().unwrap_or(1),
                                i["unit_price"].as_str().unwrap_or("0").to_string(),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();

            let current_idx = ORDER_TIMELINE
                .iter()
                .position(|s| *s == status)
                .unwrap_or(0);
            let is_terminal = status == "Delivered" || status == "Cancelled";

            let timeline: Vec<(&str, &str)> = ORDER_TIMELINE
                .iter()
                .enumerate()
                .map(|(idx, step)| {
                    let class = if idx < current_idx {
                        "step done"
                    } else if idx == current_idx {
                        "step current"
                    } else {
                        "step"
                    };
                    (class, *step)
                })
                .collect();

            let payment_status = match status.as_str() {
                "Created" => "Pending",
                "Cancelled" => "Failed",
                _ => "Succeeded",
            };
            let payment_class = match payment_status {
                "Pending" => "badge payment-pending",
                "Failed" => "badge payment-failed",
                _ => "badge payment-succeeded",
            };

            rsx! {
                div { class: "order-tracking",
                    h1 { "Order Tracking" }

                    // Payment status
                    div { class: "payment-status",
                        span { "Payment: " }
                        span { class: "{payment_class}", "{payment_status}" }
                        if payment_status == "Pending" {
                            p { class: "payment-info", "Payment processing..." }
                        }
                        if payment_status == "Failed" {
                            Link { to: Route::Checkout {}, class: "retry-btn", "Retry Payment" }
                        }
                    }

                    // Status timeline
                    div { class: "timeline",
                        for (class, step) in timeline {
                            div {
                                class: "{class}",
                                span { class: "step-dot" }
                                span { class: "step-label", "{step}" }
                            }
                        }
                    }

                    if !is_terminal {
                        button {
                            class: "refresh-btn",
                            onclick: move |_| refresh += 1,
                            "Refresh Status"
                        }
                    }

                    // Order items
                    div { class: "order-items",
                        h3 { "Items" }
                        for (name, qty, price) in items {
                            div { class: "order-item",
                                span { "{name} x{qty}" }
                                span { "${price}" }
                            }
                        }
                    }

                    // Pricing breakdown (always visible)
                    div { class: "pricing-breakdown",
                        h3 { "Open-Book Receipt" }
                        div { class: "price-line",
                            span { "Food Total" }
                            span { "${food_total}" }
                        }
                        div { class: "price-line",
                            span { "Delivery Fee" }
                            span { "${delivery_fee}" }
                        }
                        div { class: "price-line",
                            span { "Tip" }
                            span { "${tip}" }
                        }
                        div { class: "price-line",
                            span { "Federal Fee" }
                            span { "${federal_fee}" }
                        }
                        div { class: "price-line",
                            span { "Local Ops Fee" }
                            span { "${local_ops_fee}" }
                        }
                        div { class: "price-line",
                            span { "Processing (Stripe)" }
                            span { "${processing_fee}" }
                        }
                    }
                }
            }
        }
        Some(Err(e)) => rsx! {
            h1 { "Order Tracking" }
            p { class: "error", "Error: {e}" }
        },
        None => rsx! {
            h1 { "Order Tracking" }
            p { "Loading order..." }
        },
    }
}
