#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::local_db::Store;

#[component]
pub fn PublicEconomicsPage() -> Element {
    let store = use_context::<Store>();
    let economics = use_resource(move || {
        let store = store.clone();
        async move {
            crate::api::cached_get::<serde_json::Value>(
                "/public/economics",
                store.as_ref(),
                "economics",
            )
            .await
        }
    });

    rsx! {
        div { class: "economics-page",
            div { class: "hero",
                h1 { "Open-Book Economics" }
                p { class: "subtitle",
                    "Every dollar traced. No hidden fees. See exactly where your money goes."
                }
            }

            match &*economics.read_unchecked() {
                Some(Ok(data)) => {
                    let total_orders = data["total_orders"].as_i64().unwrap_or(0);
                    let food = data["total_food_revenue"].as_str().unwrap_or("0.00").to_string();
                    let delivery = data["total_delivery_fees"].as_str().unwrap_or("0.00").to_string();
                    let federal = data["total_federal_fees"].as_str().unwrap_or("0.00").to_string();
                    let local_ops = data["total_local_ops_fees"].as_str().unwrap_or("0.00").to_string();
                    let processing = data["total_processing_fees"].as_str().unwrap_or("0.00").to_string();
                    let avg = data["avg_order_value"].as_str().unwrap_or("0.00").to_string();

                    rsx! {
                        div { class: "stats-grid",
                            div { class: "stat-card",
                                h3 { "{total_orders}" }
                                p { "Total Orders" }
                            }
                            div { class: "stat-card",
                                h3 { "${avg}" }
                                p { "Avg Order Value" }
                            }
                        }

                        div { class: "pricing-breakdown",
                            h3 { "Where Your Money Goes" }
                            div { class: "price-line",
                                span { "Food Revenue (100% to restaurants)" }
                                span { "${food}" }
                            }
                            div { class: "price-line",
                                span { "Delivery Fees (100% to couriers)" }
                                span { "${delivery}" }
                            }
                            div { class: "price-line",
                                span { "Federal Fee ($1/order — protocol & security)" }
                                span { "${federal}" }
                            }
                            div { class: "price-line",
                                span { "Local Ops Fee (node operator costs)" }
                                span { "${local_ops}" }
                            }
                            div { class: "price-line",
                                span { "Processing (Stripe pass-through)" }
                                span { "${processing}" }
                            }
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    p { class: "error", "Failed to load economics data: {e}" }
                },
                None => rsx! {
                    p { "Loading economics data..." }
                },
            }
        }
    }
}
