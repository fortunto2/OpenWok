#![allow(non_snake_case)]

use dioxus::prelude::*;

#[component]
pub fn PublicEconomicsPage() -> Element {
    let economics =
        use_resource(|| async move { crate::server_fns::config::get_economics().await });

    rsx! {
        div { class: "economics-page",
            div { class: "hero",
                h1 { "Open-Book Economics" }
                p { class: "subtitle",
                    "Every dollar traced. No hidden fees. See exactly where your money goes."
                }
            }

            match &*economics.read_unchecked() {
                Some(Ok(data)) => rsx! {
                    div { class: "stats-grid",
                        div { class: "stat-card",
                            h3 { "{data.total_orders}" }
                            p { "Total Orders" }
                        }
                        div { class: "stat-card",
                            h3 { "${data.avg_order_value}" }
                            p { "Avg Order Value" }
                        }
                    }

                    div { class: "pricing-breakdown",
                        h3 { "Where Your Money Goes" }
                        div { class: "price-line",
                            span { "Food Revenue (100% to restaurants)" }
                            span { "${data.total_food_revenue}" }
                        }
                        div { class: "price-line",
                            span { "Delivery Fees (100% to couriers)" }
                            span { "${data.total_delivery_fees}" }
                        }
                        div { class: "price-line",
                            span { "Federal Fee ($1/order - protocol & security)" }
                            span { "${data.total_federal_fees}" }
                        }
                        div { class: "price-line",
                            span { "Local Ops Fee (node operator costs)" }
                            span { "${data.total_local_ops_fees}" }
                        }
                        div { class: "price-line",
                            span { "Processing (Stripe pass-through)" }
                            span { "${data.total_processing_fees}" }
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
