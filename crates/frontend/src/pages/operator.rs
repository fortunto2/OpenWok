#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::api::{
    assign_courier, fetch_admin_disputes, fetch_admin_metrics, fetch_admin_users, fetch_all_orders,
    fetch_dashboard, resolve_dispute, toggle_user_blocked, transition_order,
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

            div { class: "flex items-center gap-1 border-b border-gray-200 mb-6",
                button {
                    class: if *active_tab.read() == "overview" { "px-4 py-2 text-sm font-medium border-b-2 border-orange-500 text-orange-600" } else { "px-4 py-2 text-sm font-medium border-b-2 border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300" },
                    onclick: move |_| active_tab.set("overview".to_string()),
                    "Overview"
                }
                button {
                    class: if *active_tab.read() == "metrics" { "px-4 py-2 text-sm font-medium border-b-2 border-orange-500 text-orange-600" } else { "px-4 py-2 text-sm font-medium border-b-2 border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300" },
                    onclick: move |_| active_tab.set("metrics".to_string()),
                    "Metrics"
                }
                button {
                    class: if *active_tab.read() == "users" { "px-4 py-2 text-sm font-medium border-b-2 border-orange-500 text-orange-600" } else { "px-4 py-2 text-sm font-medium border-b-2 border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300" },
                    onclick: move |_| active_tab.set("users".to_string()),
                    "Users"
                }
                button {
                    class: if *active_tab.read() == "disputes" { "px-4 py-2 text-sm font-medium border-b-2 border-orange-500 text-orange-600" } else { "px-4 py-2 text-sm font-medium border-b-2 border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300" },
                    onclick: move |_| active_tab.set("disputes".to_string()),
                    "Disputes"
                }
                button {
                    class: "ml-auto px-3 py-1.5 text-sm text-gray-500 border border-gray-300 rounded hover:bg-gray-50",
                    onclick: move |_| refresh += 1,
                    "Refresh"
                }
            }

            if *active_tab.read() == "metrics" {
                MetricsPanel {}
            }

            if *active_tab.read() == "users" {
                UsersPanel { refresh: refresh }
            }

            if *active_tab.read() == "disputes" {
                DisputesPanel { refresh: refresh }
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
fn UsersPanel(refresh: Signal<u32>) -> Element {
    let users = use_resource(move || {
        let _ = refresh();
        fetch_admin_users()
    });

    match &*users.read_unchecked() {
        Some(Ok(list)) => {
            rsx! {
                div { class: "console-section",
                    h2 { "Users ({list.len()})" }
                    table { class: "admin-table",
                        thead {
                            tr {
                                th { "Email" }
                                th { "Name" }
                                th { "Role" }
                                th { "Blocked" }
                                th { "Action" }
                            }
                        }
                        tbody {
                            for user in list.iter() {
                                {
                                    let uid = user["id"].as_str().unwrap_or("").to_string();
                                    let email = user["email"].as_str().unwrap_or("").to_string();
                                    let name = user["name"].as_str().unwrap_or("-").to_string();
                                    let role = user["role"].as_str().unwrap_or("Customer").to_string();
                                    let blocked = user["blocked"].as_bool().unwrap_or(false);
                                    rsx! {
                                        tr { key: "{uid}",
                                            td { "{email}" }
                                            td { "{name}" }
                                            td { span { class: "badge", "{role}" } }
                                            td {
                                                if blocked {
                                                    span { class: "badge badge-danger", "Blocked" }
                                                } else {
                                                    span { class: "badge badge-success", "Active" }
                                                }
                                            }
                                            td {
                                                button {
                                                    class: "action-btn",
                                                    onclick: {
                                                        let uid = uid.clone();
                                                        let new_blocked = !blocked;
                                                        let mut refresh = refresh;
                                                        move |_| {
                                                            let uid = uid.clone();
                                                            spawn(async move {
                                                                let _ = toggle_user_blocked(&uid, new_blocked).await;
                                                                refresh += 1;
                                                            });
                                                        }
                                                    },
                                                    if blocked { "Unblock" } else { "Block" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(Err(e)) => rsx! {
            p { class: "error", "Failed to load users: {e}" }
        },
        None => rsx! {
            p { "Loading users..." }
        },
    }
}

#[component]
fn DisputesPanel(refresh: Signal<u32>) -> Element {
    let disputes = use_resource(move || {
        let _ = refresh();
        fetch_admin_disputes()
    });

    match &*disputes.read_unchecked() {
        Some(Ok(list)) => {
            rsx! {
                div { class: "console-section",
                    h2 { "Disputes ({list.len()})" }
                    if list.is_empty() {
                        p { "No disputes" }
                    }
                    for dispute in list.iter() {
                        {
                            let did = dispute["id"].as_str().unwrap_or("").to_string();
                            let order_id = dispute["order_id"].as_str().unwrap_or("").to_string();
                            let reason = dispute["reason"].as_str().unwrap_or("").to_string();
                            let status = dispute["status"].as_str().unwrap_or("Open").to_string();
                            let resolution = dispute["resolution"].as_str().unwrap_or("").to_string();
                            let is_open = status == "Open";
                            rsx! {
                                div { class: "dispute-card", key: "{did}",
                                    div { class: "dispute-header",
                                        span { class: "order-id", "Order: {order_id}" }
                                        span { class: "badge",
                                            class: if is_open { "badge-warning" } else { "" },
                                            "{status}"
                                        }
                                    }
                                    p { class: "dispute-reason", "{reason}" }
                                    if !resolution.is_empty() {
                                        p { class: "dispute-resolution", "Resolution: {resolution}" }
                                    }
                                    if is_open {
                                        div { class: "dispute-actions",
                                            button {
                                                class: "action-btn",
                                                onclick: {
                                                    let did = did.clone();
                                                    let mut refresh = refresh;
                                                    move |_| {
                                                        let did = did.clone();
                                                        spawn(async move {
                                                            let _ = resolve_dispute(&did, "Resolved", Some("resolved by operator")).await;
                                                            refresh += 1;
                                                        });
                                                    }
                                                },
                                                "Resolve"
                                            }
                                            button {
                                                class: "action-btn action-btn-secondary",
                                                onclick: {
                                                    let did = did.clone();
                                                    let mut refresh = refresh;
                                                    move |_| {
                                                        let did = did.clone();
                                                        spawn(async move {
                                                            let _ = resolve_dispute(&did, "Dismissed", None).await;
                                                            refresh += 1;
                                                        });
                                                    }
                                                },
                                                "Dismiss"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(Err(e)) => rsx! {
            p { class: "error", "Failed to load disputes: {e}" }
        },
        None => rsx! {
            p { "Loading disputes..." }
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
