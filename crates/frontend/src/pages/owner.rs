#![allow(non_snake_case)]

use dioxus::prelude::*;
use gloo_net::http::Request;
use openwok_core::types::Restaurant;

use crate::api::{API_BASE, api_patch_json, fetch_my_orders};
use crate::app::Route;
use crate::state::UserState;

#[component]
pub fn MyRestaurants() -> Element {
    let user_state = use_context::<Signal<UserState>>();
    let jwt = user_state.read().jwt.clone();

    let restaurants = use_resource(move || {
        let jwt = jwt.clone();
        async move {
            let Some(token) = jwt else {
                return Vec::new();
            };
            let resp = Request::get(&format!("{API_BASE}/my/restaurants"))
                .header("Authorization", &format!("Bearer {token}"))
                .send()
                .await;
            match resp {
                Ok(r) if r.ok() => r.json::<Vec<Restaurant>>().await.unwrap_or_default(),
                _ => Vec::new(),
            }
        }
    });

    rsx! {
        div { class: "page my-restaurants",
            div { class: "page-header",
                h1 { "My Restaurants" }
                Link { to: Route::OnboardRestaurant {}, class: "cta", "Add Restaurant" }
            }
            match &*restaurants.read() {
                Some(list) if !list.is_empty() => rsx! {
                    div { class: "restaurant-grid",
                        for r in list {
                            div { class: "restaurant-card",
                                h3 { "{r.name}" }
                                p { class: if r.active { "status active" } else { "status inactive" },
                                    if r.active { "Active" } else { "Inactive" }
                                }
                                p { "{r.menu.len()} menu items" }
                                if let Some(ref desc) = r.description {
                                    p { class: "desc", "{desc}" }
                                }
                                Link {
                                    to: Route::RestaurantSettings { id: r.id.to_string() },
                                    class: "settings-link",
                                    "Manage →"
                                }
                            }
                        }
                    }
                },
                Some(_) => rsx! {
                    p { "No restaurants yet. Create your first one!" }
                },
                None => rsx! {
                    p { "Loading..." }
                },
            }
        }
    }
}

#[component]
pub fn OnboardRestaurant() -> Element {
    let user_state = use_context::<Signal<UserState>>();
    let nav = navigator();
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut address = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut zone_id = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    rsx! {
        div { class: "page onboard",
            h1 { "Register Your Restaurant" }
            form {
                class: "onboard-form",
                onsubmit: move |evt| {
                    evt.stop_propagation();
                    let jwt = user_state.read().jwt.clone();
                    let n = name.read().clone();
                    let d = description.read().clone();
                    let a = address.read().clone();
                    let p = phone.read().clone();
                    let z = zone_id.read().clone();
                    let nav = nav;
                    submitting.set(true);
                    error.set(None);
                    spawn(async move {
                        let Some(token) = jwt else {
                            error.set(Some("Please sign in first".into()));
                            submitting.set(false);
                            return;
                        };
                        let body = serde_json::json!({
                            "name": n,
                            "zone_id": z,
                            "menu": [],
                            "description": if d.is_empty() { None } else { Some(d) },
                            "address": if a.is_empty() { None } else { Some(a) },
                            "phone": if p.is_empty() { None } else { Some(p) },
                        });
                        let resp = Request::post(&format!("{API_BASE}/restaurants"))
                            .header("Authorization", &format!("Bearer {token}"))
                            .header("Content-Type", "application/json")
                            .body(body.to_string())
                            .unwrap()
                            .send()
                            .await;
                        match resp {
                            Ok(r) if r.ok() => {
                                if let Ok(rest) = r.json::<Restaurant>().await {
                                    nav.push(Route::RestaurantSettings { id: rest.id.to_string() });
                                }
                            }
                            Ok(r) => {
                                let msg = r.text().await.unwrap_or("Failed to create restaurant".into());
                                error.set(Some(msg));
                            }
                            Err(e) => error.set(Some(e.to_string())),
                        }
                        submitting.set(false);
                    });
                },
                div { class: "form-group",
                    label { "Restaurant Name *" }
                    input {
                        r#type: "text",
                        required: true,
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { "Zone ID *" }
                    input {
                        r#type: "text",
                        required: true,
                        placeholder: "UUID of the zone",
                        value: "{zone_id}",
                        oninput: move |e| zone_id.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { "Description" }
                    textarea {
                        value: "{description}",
                        oninput: move |e| description.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { "Address" }
                    input {
                        r#type: "text",
                        value: "{address}",
                        oninput: move |e| address.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { "Phone" }
                    input {
                        r#type: "tel",
                        value: "{phone}",
                        oninput: move |e| phone.set(e.value()),
                    }
                }
                if let Some(ref err) = *error.read() {
                    p { class: "error", "{err}" }
                }
                button {
                    r#type: "submit",
                    class: "cta",
                    disabled: *submitting.read(),
                    if *submitting.read() { "Creating..." } else { "Create Restaurant" }
                }
            }
        }
    }
}

#[component]
pub fn RestaurantSettings(id: String) -> Element {
    let user_state = use_context::<Signal<UserState>>();
    let jwt = user_state.read().jwt.clone();
    let id_clone = id.clone();

    let restaurant = use_resource(move || {
        let id = id_clone.clone();
        async move {
            let resp = Request::get(&format!("{API_BASE}/restaurants/{id}"))
                .send()
                .await;
            match resp {
                Ok(r) if r.ok() => r.json::<Restaurant>().await.ok(),
                _ => None,
            }
        }
    });

    let mut active_tab = use_signal(|| "info".to_string());
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut address = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut initialized = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut save_msg = use_signal(|| None::<String>);

    // Menu item add state
    let mut new_item_name = use_signal(String::new);
    let mut new_item_price = use_signal(String::new);
    let mut adding_item = use_signal(|| false);

    // Orders state
    let mut orders = use_signal(Vec::<serde_json::Value>::new);
    let mut orders_loaded = use_signal(|| false);
    let mut prev_order_count = use_signal(|| 0_usize);
    let mut has_new_orders = use_signal(|| false);

    // Initialize form values from loaded restaurant
    if let Some(Some(ref r)) = *restaurant.read()
        && !*initialized.read()
    {
        name.set(r.name.clone());
        description.set(r.description.clone().unwrap_or_default());
        address.set(r.address.clone().unwrap_or_default());
        phone.set(r.phone.clone().unwrap_or_default());
        initialized.set(true);
    }

    // Auto-refresh orders every 15 seconds
    let id_for_poll = id.clone();
    use_future(move || {
        let rest_id = id_for_poll.clone();
        async move {
            loop {
                if let Ok(all_orders) = fetch_my_orders().await {
                    let filtered: Vec<serde_json::Value> = all_orders
                        .into_iter()
                        .filter(|o| o["restaurant_id"].as_str() == Some(&rest_id))
                        .collect();
                    let new_count = filtered.len();
                    let old_count = *prev_order_count.read();
                    if *orders_loaded.read() && new_count > old_count {
                        has_new_orders.set(true);
                    }
                    prev_order_count.set(new_count);
                    orders.set(filtered);
                    orders_loaded.set(true);
                }
                gloo_timers::future::TimeoutFuture::new(15_000).await;
            }
        }
    });

    rsx! {
        div { class: "page restaurant-settings",
            match &*restaurant.read() {
                Some(Some(r)) => {
                    let rest_id = id.clone();
                    let rest_active = r.active;
                    let menu_items = r.menu.clone();
                    let current_tab = active_tab.read().clone();
                    rsx! {
                        h1 { "Settings: {r.name}" }
                        // Tab navigation
                        div { class: "tab-nav",
                            button {
                                class: if current_tab == "info" { "tab active" } else { "tab" },
                                onclick: move |_| active_tab.set("info".into()),
                                "Info"
                            }
                            button {
                                class: if current_tab == "menu" { "tab active" } else { "tab" },
                                onclick: move |_| active_tab.set("menu".into()),
                                "Menu"
                            }
                            button {
                                class: if current_tab == "orders" { "tab active" } else { "tab" },
                                onclick: move |_| {
                                    active_tab.set("orders".into());
                                    has_new_orders.set(false);
                                },
                                "Orders"
                                if *has_new_orders.read() && current_tab != "orders" {
                                    span { class: "badge new-badge", "New" }
                                }
                            }
                        }

                        // Info tab
                        if current_tab == "info" {
                            div { class: "settings-section",
                                h2 { "Restaurant Info" }
                                div { class: "form-group",
                                    label { "Name" }
                                    input {
                                        r#type: "text",
                                        value: "{name}",
                                        oninput: move |e| name.set(e.value()),
                                    }
                                }
                                div { class: "form-group",
                                    label { "Description" }
                                    textarea {
                                        value: "{description}",
                                        oninput: move |e| description.set(e.value()),
                                    }
                                }
                                div { class: "form-group",
                                    label { "Address" }
                                    input {
                                        r#type: "text",
                                        value: "{address}",
                                        oninput: move |e| address.set(e.value()),
                                    }
                                }
                                div { class: "form-group",
                                    label { "Phone" }
                                    input {
                                        r#type: "tel",
                                        value: "{phone}",
                                        oninput: move |e| phone.set(e.value()),
                                    }
                                }
                                button {
                                    class: "cta",
                                    disabled: *saving.read(),
                                    onclick: {
                                        let rest_id = rest_id.clone();
                                        let jwt = jwt.clone();
                                        move |_| {
                                            let n = name.read().clone();
                                            let d = description.read().clone();
                                            let a = address.read().clone();
                                            let p = phone.read().clone();
                                            let rest_id = rest_id.clone();
                                            let token = jwt.clone();
                                            saving.set(true);
                                            save_msg.set(None);
                                            spawn(async move {
                                                let Some(tok) = token else { return; };
                                                let body = serde_json::json!({
                                                    "name": n,
                                                    "description": d,
                                                    "address": a,
                                                    "phone": p,
                                                });
                                                let resp = Request::patch(&format!("{API_BASE}/restaurants/{rest_id}"))
                                                    .header("Authorization", &format!("Bearer {tok}"))
                                                    .header("Content-Type", "application/json")
                                                    .body(body.to_string())
                                                    .unwrap()
                                                    .send()
                                                    .await;
                                                match resp {
                                                    Ok(r) if r.ok() => save_msg.set(Some("Saved!".into())),
                                                    _ => save_msg.set(Some("Failed to save".into())),
                                                }
                                                saving.set(false);
                                            });
                                        }
                                    },
                                    if *saving.read() { "Saving..." } else { "Save Changes" }
                                }
                                if let Some(ref msg) = *save_msg.read() {
                                    p { class: "save-msg", "{msg}" }
                                }
                                // Active toggle
                                div { class: "status-section",
                                    h3 { "Status" }
                                    p { "Currently: " strong { if rest_active { "Active" } else { "Inactive" } } }
                                    button {
                                        class: if rest_active { "btn-danger" } else { "cta" },
                                        onclick: {
                                            let rest_id = rest_id.clone();
                                            let jwt = jwt.clone();
                                            move |_| {
                                                let rest_id = rest_id.clone();
                                                let token = jwt.clone();
                                                let new_active = !rest_active;
                                                spawn(async move {
                                                    let Some(tok) = token else { return; };
                                                    let body = serde_json::json!({ "active": new_active });
                                                    let _ = Request::patch(&format!("{API_BASE}/restaurants/{rest_id}/active"))
                                                        .header("Authorization", &format!("Bearer {tok}"))
                                                        .header("Content-Type", "application/json")
                                                        .body(body.to_string())
                                                        .unwrap()
                                                        .send()
                                                        .await;
                                                    if let Some(window) = web_sys::window() {
                                                        let _ = window.location().reload();
                                                    }
                                                });
                                            }
                                        },
                                        if rest_active { "Deactivate" } else { "Activate" }
                                    }
                                }
                            }
                        }

                        // Menu tab
                        if current_tab == "menu" {
                            div { class: "settings-section",
                                h2 { "Menu ({menu_items.len()} items)" }
                                for item in &menu_items {
                                    div { class: "menu-item-row",
                                        span { class: "item-name", "{item.name}" }
                                        span { class: "item-price", "${item.price}" }
                                        button {
                                            class: "btn-small btn-danger",
                                            onclick: {
                                                let item_id = item.id.to_string();
                                                let jwt = jwt.clone();
                                                move |_| {
                                                    let item_id = item_id.clone();
                                                    let token = jwt.clone();
                                                    spawn(async move {
                                                        let Some(tok) = token else { return; };
                                                        let _ = Request::delete(&format!("{API_BASE}/menu-items/{item_id}"))
                                                            .header("Authorization", &format!("Bearer {tok}"))
                                                            .send()
                                                            .await;
                                                        if let Some(window) = web_sys::window() {
                                                            let _ = window.location().reload();
                                                        }
                                                    });
                                                }
                                            },
                                            "Delete"
                                        }
                                    }
                                }
                                div { class: "add-item-form",
                                    h3 { "Add Menu Item" }
                                    input {
                                        r#type: "text",
                                        placeholder: "Item name",
                                        value: "{new_item_name}",
                                        oninput: move |e| new_item_name.set(e.value()),
                                    }
                                    input {
                                        r#type: "text",
                                        placeholder: "Price (e.g. 12.99)",
                                        value: "{new_item_price}",
                                        oninput: move |e| new_item_price.set(e.value()),
                                    }
                                    button {
                                        class: "cta",
                                        disabled: *adding_item.read(),
                                        onclick: {
                                            let rest_id = rest_id.clone();
                                            let jwt = jwt.clone();
                                            move |_| {
                                                let n = new_item_name.read().clone();
                                                let p = new_item_price.read().clone();
                                                let rest_id = rest_id.clone();
                                                let token = jwt.clone();
                                                if n.is_empty() || p.is_empty() { return; }
                                                adding_item.set(true);
                                                spawn(async move {
                                                    let Some(tok) = token else { return; };
                                                    let body = serde_json::json!({
                                                        "name": n,
                                                        "price": p,
                                                    });
                                                    let _ = Request::post(&format!("{API_BASE}/restaurants/{rest_id}/menu"))
                                                        .header("Authorization", &format!("Bearer {tok}"))
                                                        .header("Content-Type", "application/json")
                                                        .body(body.to_string())
                                                        .unwrap()
                                                        .send()
                                                        .await;
                                                    adding_item.set(false);
                                                    if let Some(window) = web_sys::window() {
                                                        let _ = window.location().reload();
                                                    }
                                                });
                                            }
                                        },
                                        if *adding_item.read() { "Adding..." } else { "Add Item" }
                                    }
                                }
                            }
                        }

                        // Orders tab
                        if current_tab == "orders" {
                            div { class: "settings-section",
                                h2 { "Orders" }
                                if !*orders_loaded.read() {
                                    p { "Loading orders..." }
                                } else if orders.read().is_empty() {
                                    p { "No orders yet." }
                                } else {
                                    div { class: "restaurant-list",
                                        for order in orders.read().iter() {
                                            {
                                                let oid_full = order["id"].as_str().unwrap_or("").to_string();
                                                let oid_short = if oid_full.len() > 8 { &oid_full[..8] } else { &oid_full };
                                                let status = order["status"].as_str().unwrap_or("Unknown").to_string();
                                                let customer_addr = order["customer_address"].as_str().unwrap_or("—").to_string();
                                                let food_total = order["pricing"]["food_total"].as_str().unwrap_or("0").to_string();
                                                let created = order["created_at"].as_str().unwrap_or("").to_string();
                                                let created_short = if created.len() > 16 { &created[..16] } else { &created };
                                                let status_class = match status.as_str() {
                                                    "Confirmed" => "badge confirmed",
                                                    "Preparing" => "badge preparing",
                                                    "ReadyForPickup" => "badge ready",
                                                    "InDelivery" => "badge active",
                                                    "Delivered" => "badge delivered",
                                                    "Cancelled" => "badge cancelled",
                                                    _ => "badge",
                                                };

                                                // Items list
                                                let items = order["items"].as_array();

                                                rsx! {
                                                    div { class: "restaurant-card order-card",
                                                        h3 {
                                                            "#{oid_short}"
                                                            " "
                                                            span { class: "{status_class}", "{status}" }
                                                        }
                                                        if let Some(items_arr) = items {
                                                            ul { class: "order-items",
                                                                for item in items_arr {
                                                                    li {
                                                                        "{item[\"quantity\"].as_u64().unwrap_or(1)}x {item[\"name\"].as_str().unwrap_or(\"—\")}"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        p { "Total: ${food_total}" }
                                                        p { class: "order-address", "Deliver to: {customer_addr}" }
                                                        p { class: "order-time", "{created_short}" }

                                                        // Action buttons based on status
                                                        div { class: "order-actions",
                                                            if status == "Confirmed" {
                                                                {
                                                                    let oid = oid_full.clone();
                                                                    let rid = rest_id.clone();
                                                                    rsx! {
                                                                        button {
                                                                            class: "cta",
                                                                            onclick: move |_| {
                                                                                let oid = oid.clone();
                                                                                let rid = rid.clone();
                                                                                spawn(async move {
                                                                                    let _ = api_patch_json(
                                                                                        &format!("/orders/{oid}/status"),
                                                                                        &serde_json::json!({"status": "Preparing"}),
                                                                                    ).await;
                                                                                    if let Ok(all) = fetch_my_orders().await {
                                                                                        orders.set(all.into_iter().filter(|o| o["restaurant_id"].as_str() == Some(&rid)).collect());
                                                                                    }
                                                                                });
                                                                            },
                                                                            "Accept"
                                                                        }
                                                                    }
                                                                }
                                                                {
                                                                    let oid = oid_full.clone();
                                                                    let rid = rest_id.clone();
                                                                    rsx! {
                                                                        button {
                                                                            class: "btn-danger",
                                                                            onclick: move |_| {
                                                                                let oid = oid.clone();
                                                                                let rid = rid.clone();
                                                                                spawn(async move {
                                                                                    let _ = api_patch_json(
                                                                                        &format!("/orders/{oid}/status"),
                                                                                        &serde_json::json!({"status": "Cancelled"}),
                                                                                    ).await;
                                                                                    if let Ok(all) = fetch_my_orders().await {
                                                                                        orders.set(all.into_iter().filter(|o| o["restaurant_id"].as_str() == Some(&rid)).collect());
                                                                                    }
                                                                                });
                                                                            },
                                                                            "Cancel"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            if status == "Preparing" {
                                                                {
                                                                    let oid = oid_full.clone();
                                                                    let rid = rest_id.clone();
                                                                    rsx! {
                                                                        button {
                                                                            class: "cta",
                                                                            onclick: move |_| {
                                                                                let oid = oid.clone();
                                                                                let rid = rid.clone();
                                                                                spawn(async move {
                                                                                    let _ = api_patch_json(
                                                                                        &format!("/orders/{oid}/status"),
                                                                                        &serde_json::json!({"status": "ReadyForPickup"}),
                                                                                    ).await;
                                                                                    if let Ok(all) = fetch_my_orders().await {
                                                                                        orders.set(all.into_iter().filter(|o| o["restaurant_id"].as_str() == Some(&rid)).collect());
                                                                                    }
                                                                                });
                                                                            },
                                                                            "Mark Ready"
                                                                        }
                                                                    }
                                                                }
                                                                {
                                                                    let oid = oid_full.clone();
                                                                    let rid = rest_id.clone();
                                                                    rsx! {
                                                                        button {
                                                                            class: "btn-danger",
                                                                            onclick: move |_| {
                                                                                let oid = oid.clone();
                                                                                let rid = rid.clone();
                                                                                spawn(async move {
                                                                                    let _ = api_patch_json(
                                                                                        &format!("/orders/{oid}/status"),
                                                                                        &serde_json::json!({"status": "Cancelled"}),
                                                                                    ).await;
                                                                                    if let Ok(all) = fetch_my_orders().await {
                                                                                        orders.set(all.into_iter().filter(|o| o["restaurant_id"].as_str() == Some(&rid)).collect());
                                                                                    }
                                                                                });
                                                                            },
                                                                            "Cancel"
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
                            }
                        }
                    }
                },
                Some(None) => rsx! { p { "Restaurant not found" } },
                None => rsx! { p { "Loading..." } },
            }
        }
    }
}
