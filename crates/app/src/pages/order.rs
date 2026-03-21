#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::app::Route;

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
    let mut refresh = use_signal(|| 0u32);
    let order = use_resource(move || {
        let _ = refresh();
        let id = id.clone();
        async move { crate::server_fns::orders::get_order(id).await }
    });

    match &*order.read_unchecked() {
        Some(Ok(order)) => {
            let status = format!("{:?}", order.status);
            let current_idx = ORDER_TIMELINE
                .iter()
                .position(|step| *step == status)
                .unwrap_or(0);
            let is_terminal = status == "Delivered" || status == "Cancelled";
            let payment_status = payment_status_for_order(&status);
            let payment_class = payment_badge_class(payment_status);

            rsx! {
                div { class: "order-tracking",
                    h1 { "Order Tracking" }

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

                    div { class: "timeline",
                        for (idx, step) in ORDER_TIMELINE.iter().enumerate() {
                            div {
                                class: timeline_class(idx, current_idx),
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

                    div { class: "order-items",
                        h3 { "Items" }
                        for item in &order.items {
                            div { class: "order-item",
                                span { "{item.name} x{item.quantity}" }
                                span { "{item.unit_price}" }
                            }
                        }
                    }

                    div { class: "pricing-breakdown",
                        h3 { "Open-Book Receipt" }
                        div { class: "price-line",
                            span { "Food Total" }
                            span { "{order.pricing.food_total}" }
                        }
                        div { class: "price-line",
                            span { "Delivery Fee" }
                            span { "{order.pricing.delivery_fee}" }
                        }
                        div { class: "price-line",
                            span { "Tip" }
                            span { "{order.pricing.tip}" }
                        }
                        div { class: "price-line",
                            span { "Federal Fee" }
                            span { "{order.pricing.federal_fee}" }
                        }
                        div { class: "price-line",
                            span { "Local Ops Fee" }
                            span { "{order.pricing.local_ops_fee}" }
                        }
                        div { class: "price-line",
                            span { "Processing (Stripe)" }
                            span { "{order.pricing.processing_fee}" }
                        }
                    }
                }
            }
        }
        Some(Err(error)) => rsx! {
            h1 { "Order Tracking" }
            p { class: "error", "Error: {error}" }
        },
        None => rsx! {
            h1 { "Order Tracking" }
            p { "Loading order..." }
        },
    }
}

fn payment_status_for_order(status: &str) -> &'static str {
    match status {
        "Created" => "Pending",
        "Cancelled" => "Failed",
        _ => "Succeeded",
    }
}

fn payment_badge_class(status: &str) -> &'static str {
    match status {
        "Pending" => "badge payment-pending",
        "Failed" => "badge payment-failed",
        _ => "badge payment-succeeded",
    }
}

fn timeline_class(idx: usize, current_idx: usize) -> &'static str {
    if idx < current_idx {
        "step done"
    } else if idx == current_idx {
        "step current"
    } else {
        "step"
    }
}
