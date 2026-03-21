#![allow(non_snake_case)]

use dioxus::prelude::*;
use openwok_core::order::OrderStatus;
use openwok_core::repo::AdminMetrics;
use openwok_core::types::{Dispute, User};

#[component]
pub fn OperatorConsole() -> Element {
    let mut refresh = use_signal(|| 0u32);
    let mut active_tab = use_signal(|| "overview".to_string());

    let restaurants = use_resource(move || {
        let _ = refresh();
        async move { crate::server_fns::restaurants::get_restaurants().await }
    });
    let couriers = use_resource(move || {
        let _ = refresh();
        async move { crate::server_fns::admin::list_couriers().await }
    });
    let orders = use_resource(move || {
        let _ = refresh();
        async move { crate::server_fns::admin::list_orders().await }
    });

    let restaurant_list = read_or_default(restaurants.read_unchecked().as_ref().cloned());
    let courier_list = read_or_default(couriers.read_unchecked().as_ref().cloned());
    let order_list = read_or_default(orders.read_unchecked().as_ref().cloned());
    let couriers_online = courier_list
        .iter()
        .filter(|courier| courier.available)
        .count();

    rsx! {
        div { class: "operator-console",
            h1 { "Node Operator Console" }

            div { class: "flex items-center gap-1 border-b border-gray-200 mb-6",
                button {
                    class: tab_class(active_tab.read().as_str(), "overview"),
                    onclick: move |_| active_tab.set("overview".to_string()),
                    "Overview"
                }
                button {
                    class: tab_class(active_tab.read().as_str(), "metrics"),
                    onclick: move |_| active_tab.set("metrics".to_string()),
                    "Metrics"
                }
                button {
                    class: tab_class(active_tab.read().as_str(), "users"),
                    onclick: move |_| active_tab.set("users".to_string()),
                    "Users"
                }
                button {
                    class: tab_class(active_tab.read().as_str(), "disputes"),
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
                div { class: "stats-grid",
                    div { class: "stat-card",
                        h3 { "{restaurant_list.len()}" }
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

                div { class: "console-section",
                    h2 { "Orders" }
                    if order_list.is_empty() {
                        p { "No orders yet" }
                    }
                    for order in order_list.iter() {
                        {
                            let order_id = order.id.to_string();
                            let status = order.status;
                            let address = order.customer_address.clone();
                            let has_courier = order.courier_id.is_some();
                            rsx! {
                                OrderRow {
                                    key: "{order_id}",
                                    order_id: order_id,
                                    status: status,
                                    address: address,
                                    has_courier: has_courier,
                                    on_action: move |_| refresh += 1,
                                }
                            }
                        }
                    }
                }

                div { class: "console-section",
                    h2 { "Restaurants" }
                    for restaurant in restaurant_list {
                        div { class: "console-row", key: "{restaurant.id}",
                            span { "{restaurant.name}" }
                            span { "{restaurant.menu.len()} items" }
                        }
                    }
                }

                div { class: "console-section",
                    h2 { "Available Couriers" }
                    if courier_list.iter().all(|courier| !courier.available) {
                        p { "No couriers online" }
                    }
                    for courier in courier_list.into_iter().filter(|courier| courier.available) {
                        div { class: "console-row", key: "{courier.id}",
                            span { "{courier.name}" }
                            span { class: "badge", "Available" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MetricsPanel() -> Element {
    let metrics =
        use_resource(|| async move { crate::server_fns::config::get_admin_metrics().await });

    match &*metrics.read_unchecked() {
        Some(Ok(data)) => render_metrics(data),
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
        async move { crate::server_fns::admin::list_users().await }
    });

    match &*users.read_unchecked() {
        Some(Ok(list)) => rsx! {
            div { class: "console-section",
                h2 { "Users ({list.len()})" }
                table { class: "w-full text-sm bg-white border border-gray-200 rounded-lg overflow-hidden",
                    thead { class: "bg-gray-50 text-left",
                        tr {
                            th { class: "px-3 py-2", "Email" }
                            th { class: "px-3 py-2", "Name" }
                            th { class: "px-3 py-2", "Role" }
                            th { class: "px-3 py-2", "Blocked" }
                            th { class: "px-3 py-2", "Action" }
                        }
                    }
                    tbody {
                        for user in list {
                            UserRow {
                                key: "{user.id}",
                                user: user.clone(),
                                refresh: refresh,
                            }
                        }
                    }
                }
            }
        },
        Some(Err(e)) => rsx! {
            p { class: "error", "Failed to load users: {e}" }
        },
        None => rsx! {
            p { "Loading users..." }
        },
    }
}

#[component]
fn UserRow(user: User, refresh: Signal<u32>) -> Element {
    let blocked = user.blocked;
    let user_id = user.id.to_string();
    let email = user.email.clone();
    let display_name = user.name.clone().unwrap_or_else(|| "-".to_string());
    let role = user.role.to_string();

    rsx! {
        tr { class: "border-t border-gray-100",
            td { class: "px-3 py-2", "{email}" }
            td { class: "px-3 py-2", "{display_name}" }
            td { class: "px-3 py-2",
                span { class: "badge", "{role}" }
            }
            td { class: "px-3 py-2",
                if blocked {
                    span { class: "badge bg-red-100 text-red-700", "Blocked" }
                } else {
                    span { class: "badge bg-green-100 text-green-700", "Active" }
                }
            }
            td { class: "px-3 py-2",
                button {
                    class: "action-btn",
                    onclick: move |_| {
                        let user_id = user_id.clone();
                        let mut refresh = refresh;
                        spawn(async move {
                            let _ = crate::server_fns::admin::set_user_blocked(user_id, !blocked).await;
                            refresh += 1;
                        });
                    },
                    if blocked { "Unblock" } else { "Block" }
                }
            }
        }
    }
}

#[component]
fn DisputesPanel(refresh: Signal<u32>) -> Element {
    let disputes = use_resource(move || {
        let _ = refresh();
        async move { crate::server_fns::admin::list_disputes().await }
    });

    match &*disputes.read_unchecked() {
        Some(Ok(list)) => rsx! {
            div { class: "console-section",
                h2 { "Disputes ({list.len()})" }
                if list.is_empty() {
                    p { "No disputes" }
                }
                for dispute in list {
                    DisputeCard {
                        key: "{dispute.id}",
                        dispute: dispute.clone(),
                        refresh: refresh,
                    }
                }
            }
        },
        Some(Err(e)) => rsx! {
            p { class: "error", "Failed to load disputes: {e}" }
        },
        None => rsx! {
            p { "Loading disputes..." }
        },
    }
}

#[component]
fn DisputeCard(dispute: Dispute, refresh: Signal<u32>) -> Element {
    let dispute_id = dispute.id.to_string();
    let is_open = matches!(dispute.status, openwok_core::types::DisputeStatus::Open);
    let resolve_dispute_id = dispute_id.clone();
    let dismiss_dispute_id = dispute_id.clone();

    rsx! {
        div { class: "bg-white border border-gray-200 rounded-lg p-4 mb-3",
            div { class: "flex items-center justify-between gap-3 mb-2",
                span { class: "order-id", "Order: {dispute.order_id}" }
                span { class: "badge", "{dispute.status}" }
            }
            p { class: "mb-2", "{dispute.reason}" }
            if let Some(resolution) = &dispute.resolution {
                p { class: "text-sm text-gray-500 mb-2", "Resolution: {resolution}" }
            }
            if is_open {
                div { class: "flex gap-2",
                    button {
                        class: "action-btn",
                        onclick: move |_| {
                            let dispute_id = resolve_dispute_id.clone();
                            let mut refresh = refresh;
                            spawn(async move {
                                let _ = crate::server_fns::admin::resolve_dispute(
                                    dispute_id,
                                    "Resolved".to_string(),
                                    Some("resolved by operator".to_string()),
                                )
                                .await;
                                refresh += 1;
                            });
                        },
                        "Resolve"
                    }
                    button {
                        class: "px-4 py-2 text-sm border border-gray-300 rounded hover:bg-gray-50",
                        onclick: move |_| {
                            let dispute_id = dismiss_dispute_id.clone();
                            let mut refresh = refresh;
                            spawn(async move {
                                let _ = crate::server_fns::admin::resolve_dispute(
                                    dispute_id,
                                    "Dismissed".to_string(),
                                    None,
                                )
                                .await;
                                refresh += 1;
                            });
                        },
                        "Dismiss"
                    }
                }
            }
        }
    }
}

#[component]
fn OrderRow(
    order_id: String,
    status: OrderStatus,
    address: String,
    has_courier: bool,
    on_action: EventHandler<()>,
) -> Element {
    let needs_assign = !has_courier
        && matches!(
            status,
            OrderStatus::Created | OrderStatus::Confirmed | OrderStatus::Preparing
        );
    let next_status = next_order_status(status);
    let status_label = format!("{:?}", status);
    let assign_order_id = order_id.clone();
    let transition_order_id = order_id.clone();

    rsx! {
        div { class: "order-row",
            div { class: "order-row-info",
                span { class: "order-id", "{order_id}" }
                span { class: "order-status badge", "{status_label}" }
                span { class: "order-addr", "{address}" }
            }
            div { class: "order-row-actions",
                if needs_assign {
                    button {
                        class: "action-btn",
                        onclick: move |_| {
                            let order_id = assign_order_id.clone();
                            let handler = on_action;
                            spawn(async move {
                                let _ = crate::server_fns::admin::assign_courier(order_id).await;
                                handler.call(());
                            });
                        },
                        "Assign Courier"
                    }
                }
                if let Some(next_status) = next_status {
                    button {
                        class: "action-btn",
                        onclick: move |_| {
                            let order_id = transition_order_id.clone();
                            let handler = on_action;
                            let next_status = next_status.to_string();
                            spawn(async move {
                                let _ = crate::server_fns::admin::transition_order_status(order_id, next_status).await;
                                handler.call(());
                            });
                        },
                        "-> {next_status}"
                    }
                }
            }
        }
    }
}

fn render_metrics(data: &AdminMetrics) -> Element {
    let util_pct = if data.courier_utilization.total > 0 {
        format!(
            "{:.0}%",
            (data.courier_utilization.available as f64 / data.courier_utilization.total as f64)
                * 100.0
        )
    } else {
        "N/A".to_string()
    };

    rsx! {
        div { class: "metrics-panel",
            div { class: "stats-grid",
                div { class: "stat-card",
                    h3 { "{data.order_count}" }
                    p { "Total Orders" }
                }
                div { class: "stat-card",
                    h3 { "{data.on_time_delivery_rate:.1}%" }
                    p { "On-Time Rate" }
                }
                div { class: "stat-card",
                    h3 { "{data.avg_eta_error_minutes:.1} min" }
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
                    span { "${data.revenue_breakdown.total_food_revenue}" }
                }
                div { class: "price-line",
                    span { "Delivery Fees" }
                    span { "${data.revenue_breakdown.total_delivery_fees}" }
                }
                div { class: "price-line",
                    span { "Federal Fees" }
                    span { "${data.revenue_breakdown.total_federal_fees}" }
                }
                div { class: "price-line",
                    span { "Local Ops Fees" }
                    span { "${data.revenue_breakdown.total_local_ops_fees}" }
                }
                div { class: "price-line",
                    span { "Processing Fees" }
                    span { "${data.revenue_breakdown.total_processing_fees}" }
                }
            }

            if !data.orders_by_zone.is_empty() {
                div { class: "console-section",
                    h2 { "Orders by Zone" }
                    for (zone_name, count) in data.orders_by_zone.iter() {
                        div { class: "console-row", key: "{zone_name}",
                            span { "{zone_name}" }
                            span { "{count} orders" }
                        }
                    }
                }
            }
        }
    }
}

fn next_order_status(status: OrderStatus) -> Option<&'static str> {
    match status {
        OrderStatus::Created => Some("Confirmed"),
        OrderStatus::Confirmed => Some("Preparing"),
        OrderStatus::Preparing => Some("ReadyForPickup"),
        OrderStatus::ReadyForPickup => Some("InDelivery"),
        OrderStatus::InDelivery => Some("Delivered"),
        OrderStatus::Delivered | OrderStatus::Cancelled => None,
    }
}

fn read_or_default<T: Clone>(value: Option<Result<Vec<T>, ServerFnError>>) -> Vec<T> {
    match value {
        Some(Ok(items)) => items.clone(),
        Some(Err(_)) | None => Vec::new(),
    }
}

fn tab_class(active: &str, tab: &str) -> &'static str {
    if active == tab {
        "px-4 py-2 text-sm font-medium border-b-2 border-orange-500 text-orange-600"
    } else {
        "px-4 py-2 text-sm font-medium border-b-2 border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
    }
}
