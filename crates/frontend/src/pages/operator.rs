#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::api::{
    assign_courier, fetch_admin_metrics, fetch_all_orders, fetch_dashboard, transition_order,
};

#[component]
pub fn OperatorConsole() -> Element {
    let mut refresh = use_signal(|| 0u32);
    let mut active_tab = use_signal(|| "overview".to_string());
    let dashboard = use_resource(move || {
        let _ = refresh();
        fetch_dashboard()
    });
    let orders = use_resource(move || {
        let _ = refresh();
        fetch_all_orders()
    });

    let dashboard_data = dashboard.read_unchecked();
    let orders_data = orders.read_unchecked();

    let (restaurant_count, couriers_online, restaurants, couriers) = match &*dashboard_data {
        Some(Ok(data)) => {
            let rc = data["restaurant_count"].as_u64().unwrap_or(0);
            let co = data["couriers_online"].as_u64().unwrap_or(0);
            let r: Vec<(String, usize)> = data["restaurants"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|r| {
                            (
                                r["name"].as_str().unwrap_or("").to_string(),
                                r["menu"].as_array().map(|m| m.len()).unwrap_or(0),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            let c: Vec<String> = data["couriers"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|c| c["name"].as_str().unwrap_or("").to_string())
                        .collect()
                })
                .unwrap_or_default();
            (rc, co, r, c)
        }
        _ => (0, 0, vec![], vec![]),
    };

    let order_list: Vec<(String, String, String, bool)> = match &*orders_data {
        Some(Ok(list)) => list
            .iter()
            .map(|o| {
                let id = o["id"].as_str().unwrap_or("").to_string();
                let status = o["status"].as_str().unwrap_or("").to_string();
                let addr = o["customer_address"].as_str().unwrap_or("").to_string();
                let has_courier = o["courier_id"].as_str().is_some();
                (id, status, addr, has_courier)
            })
            .collect(),
        _ => vec![],
    };

    rsx! {
        div { class: "operator-console",
            h1 { "Node Operator Console" }

            div { class: "tab-bar",
                button {
                    class: if *active_tab.read() == "overview" { "tab-btn active" } else { "tab-btn" },
                    onclick: move |_| active_tab.set("overview".to_string()),
                    "Overview"
                }
                button {
                    class: if *active_tab.read() == "metrics" { "tab-btn active" } else { "tab-btn" },
                    onclick: move |_| active_tab.set("metrics".to_string()),
                    "Metrics"
                }
                button {
                    class: "refresh-btn",
                    onclick: move |_| refresh += 1,
                    "Refresh"
                }
            }

            if *active_tab.read() == "metrics" {
                MetricsPanel {}
            }

            if *active_tab.read() == "overview" {

            // Stats
            div { class: "stats-grid",
                div { class: "stat-card",
                    h3 { "{restaurant_count}" }
                    p { "Restaurants" }
                }
                div { class: "stat-card",
                    h3 { "{couriers_online}" }
                    p { "Couriers Online" }
                }
                div { class: "stat-card",
                    h3 { "{order_list.len()}" }
                    p { "Orders" }
                }
            }

            // Active Orders
            div { class: "console-section",
                h2 { "Orders" }
                if order_list.is_empty() {
                    p { "No orders yet" }
                }
                for (oid, status, addr, has_courier) in order_list {
                    OrderRow {
                        order_id: oid,
                        status: status,
                        address: addr,
                        has_courier: has_courier,
                        on_action: move |_| refresh += 1,
                    }
                }
            }

            // Restaurants
            div { class: "console-section",
                h2 { "Restaurants" }
                for (name, menu_count) in restaurants {
                    div { class: "console-row",
                        span { "{name}" }
                        span { "{menu_count} items" }
                    }
                }
            }

            // Couriers
            div { class: "console-section",
                h2 { "Available Couriers" }
                if couriers.is_empty() {
                    p { "No couriers online" }
                }
                for name in couriers {
                    div { class: "console-row",
                        span { "{name}" }
                        span { class: "badge", "Available" }
                    }
                }
            }

            } // end overview tab
        }
    }
}

#[component]
fn MetricsPanel() -> Element {
    let metrics = use_resource(fetch_admin_metrics);

    match &*metrics.read_unchecked() {
        Some(Ok(data)) => {
            let order_count = data["order_count"].as_i64().unwrap_or(0);
            let on_time_rate = data["on_time_delivery_rate"].as_f64().unwrap_or(0.0);
            let eta_error = data["avg_eta_error_minutes"].as_f64().unwrap_or(0.0);
            let revenue = &data["revenue_breakdown"];
            let food_rev = revenue["total_food_revenue"]
                .as_str()
                .unwrap_or("0.00")
                .to_string();
            let delivery_rev = revenue["total_delivery_fees"]
                .as_str()
                .unwrap_or("0.00")
                .to_string();
            let federal_rev = revenue["total_federal_fees"]
                .as_str()
                .unwrap_or("0.00")
                .to_string();
            let courier_util = &data["courier_utilization"];
            let avail = courier_util["available"].as_i64().unwrap_or(0);
            let total_couriers = courier_util["total"].as_i64().unwrap_or(0);
            let util_pct = if total_couriers > 0 {
                format!("{:.0}%", (avail as f64 / total_couriers as f64) * 100.0)
            } else {
                "N/A".to_string()
            };

            let zones: Vec<(String, i64)> = data["orders_by_zone"]
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| (k.clone(), v.as_i64().unwrap_or(0)))
                        .collect()
                })
                .unwrap_or_default();

            rsx! {
                div { class: "metrics-panel",
                    div { class: "stats-grid",
                        div { class: "stat-card",
                            h3 { "{order_count}" }
                            p { "Total Orders" }
                        }
                        div { class: "stat-card",
                            h3 { "{on_time_rate:.1}%" }
                            p { "On-Time Rate" }
                        }
                        div { class: "stat-card",
                            h3 { "{eta_error:.1} min" }
                            p { "Avg ETA Error" }
                        }
                        div { class: "stat-card",
                            h3 { "{util_pct}" }
                            p { "Courier Availability" }
                        }
                    }

                    div { class: "console-section",
                        h2 { "Revenue Breakdown" }
                        div { class: "price-line",
                            span { "Food Revenue" }
                            span { "${food_rev}" }
                        }
                        div { class: "price-line",
                            span { "Delivery Fees" }
                            span { "${delivery_rev}" }
                        }
                        div { class: "price-line",
                            span { "Federal Fees" }
                            span { "${federal_rev}" }
                        }
                    }

                    if !zones.is_empty() {
                        div { class: "console-section",
                            h2 { "Orders by Zone" }
                            for (zone_name, count) in zones {
                                div { class: "console-row",
                                    span { "{zone_name}" }
                                    span { "{count} orders" }
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(Err(e)) => rsx! {
            p { class: "error", "Failed to load metrics: {e}" }
        },
        None => rsx! {
            p { "Loading metrics..." }
        },
    }
}

#[component]
fn OrderRow(
    order_id: String,
    status: String,
    address: String,
    has_courier: bool,
    on_action: EventHandler<()>,
) -> Element {
    let needs_assign =
        !has_courier && (status == "Created" || status == "Confirmed" || status == "Preparing");
    let next_status = match status.as_str() {
        "Created" => Some("Confirmed"),
        "Confirmed" => Some("Preparing"),
        "Preparing" => Some("ReadyForPickup"),
        "ReadyForPickup" => Some("InDelivery"),
        "InDelivery" => Some("Delivered"),
        _ => None,
    };

    rsx! {
        div { class: "order-row",
            div { class: "order-row-info",
                span { class: "order-id", "{order_id}" }
                span { class: "order-status badge", "{status}" }
                span { class: "order-addr", "{address}" }
            }
            div { class: "order-row-actions",
                if needs_assign {
                    button {
                        class: "action-btn",
                        onclick: {
                            let oid = order_id.clone();
                            move |_| {
                                let oid = oid.clone();
                                let handler = on_action;
                                spawn(async move {
                                    let _ = assign_courier(oid).await;
                                    handler.call(());
                                });
                            }
                        },
                        "Assign Courier"
                    }
                }
                if let Some(next) = next_status {
                    button {
                        class: "action-btn",
                        onclick: {
                            let oid = order_id.clone();
                            let ns = next.to_string();
                            move |_| {
                                let oid = oid.clone();
                                let ns = ns.clone();
                                let handler = on_action;
                                spawn(async move {
                                    let _ = transition_order(oid, ns).await;
                                    handler.call(());
                                });
                            }
                        },
                        "→ {next}"
                    }
                }
            }
        }
    }
}
