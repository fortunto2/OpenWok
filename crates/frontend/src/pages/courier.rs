#![allow(non_snake_case)]

use dioxus::prelude::*;
use gloo_net::http::Request;

use crate::api::{API_BASE, api_get, api_patch_json};
use crate::app::Route;
use crate::state::UserState;

#[component]
pub fn RegisterCourier() -> Element {
    let user_state = use_context::<Signal<UserState>>();
    let nav = navigator();
    let mut name = use_signal(String::new);
    let mut zone_id = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    // Fetch zones from restaurants (extract unique zone_id + name)
    let zones = use_resource(|| async {
        let restaurants: Vec<serde_json::Value> = api_get("/restaurants").await.ok()?;
        let mut seen = std::collections::HashMap::new();
        for r in &restaurants {
            if let (Some(zid), Some(zname)) = (r["zone_id"].as_str(), r["name"].as_str()) {
                seen.entry(zid.to_string()).or_insert_with(|| {
                    // Zone name not in restaurant data — use restaurant's zone_id
                    zid.to_string()
                });
                // We don't have zone names in the API, just use zone_id short form
                let _ = zname;
            }
        }
        Some(seen.into_iter().collect::<Vec<(String, String)>>())
    });

    rsx! {
        div { class: "page onboard",
            h1 { "Register as Courier" }
            p { "Join OpenWok's delivery network. Pick your zone and start delivering." }
            form {
                class: "onboard-form",
                onsubmit: move |evt| {
                    evt.stop_propagation();
                    let jwt = user_state.read().jwt.clone();
                    let n = name.read().clone();
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
                        });
                        let resp = Request::post(&format!("{API_BASE}/couriers"))
                            .header("Authorization", &format!("Bearer {token}"))
                            .header("Content-Type", "application/json")
                            .body(body.to_string())
                            .unwrap()
                            .send()
                            .await;
                        match resp {
                            Ok(r) if r.ok() => {
                                nav.push(Route::MyDeliveries {});
                            }
                            Ok(r) => {
                                let msg = r.text().await.unwrap_or("Failed to register".into());
                                error.set(Some(msg));
                            }
                            Err(e) => error.set(Some(e.to_string())),
                        }
                        submitting.set(false);
                    });
                },
                div { class: "form-group",
                    label { "Your Name *" }
                    input {
                        r#type: "text",
                        required: true,
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }
                div { class: "form-group",
                    label { "Delivery Zone *" }
                    match &*zones.read() {
                        Some(Some(z)) if !z.is_empty() => rsx! {
                            select {
                                required: true,
                                value: "{zone_id}",
                                onchange: move |e| zone_id.set(e.value()),
                                option { value: "", "Select a zone..." }
                                for (zid, _label) in z.iter() {
                                    option { value: "{zid}", "{zid}" }
                                }
                            }
                        },
                        _ => rsx! {
                            input {
                                r#type: "text",
                                required: true,
                                placeholder: "Zone UUID",
                                value: "{zone_id}",
                                oninput: move |e| zone_id.set(e.value()),
                            }
                        },
                    }
                }
                if let Some(ref err) = *error.read() {
                    p { class: "error", "{err}" }
                }
                button {
                    r#type: "submit",
                    class: "cta",
                    disabled: *submitting.read(),
                    if *submitting.read() { "Registering..." } else { "Register as Courier" }
                }
            }
        }
    }
}

#[component]
pub fn MyDeliveries() -> Element {
    let user_state = use_context::<Signal<UserState>>();

    // Fetch courier profile
    let courier = use_resource(move || {
        let jwt = user_state.read().jwt.clone();
        async move {
            let _jwt = jwt?;
            api_get::<serde_json::Value>("/couriers/me").await.ok()
        }
    });

    // Fetch deliveries
    let deliveries = use_resource(move || {
        let jwt = user_state.read().jwt.clone();
        async move {
            let _jwt = jwt?;
            api_get::<Vec<serde_json::Value>>("/my/deliveries")
                .await
                .ok()
        }
    });

    rsx! {
        div { class: "page my-restaurants",
            h1 { "My Deliveries" }

            match &*courier.read() {
                Some(Some(c)) => rsx! {
                    div { class: "courier-profile",
                        p { "Courier: {c[\"name\"].as_str().unwrap_or(\"—\")}" }
                        p {
                            "Status: ",
                            if c["available"].as_bool().unwrap_or(false) {
                                span { class: "badge active", "Available" }
                            } else {
                                span { class: "badge", "Busy" }
                            }
                        }
                    }
                },
                Some(None) => rsx! {
                    p { "Not registered as a courier." }
                    Link { to: Route::RegisterCourier {}, class: "cta", "Register Now" }
                },
                None => rsx! { p { "Loading..." } },
            }

            match &*deliveries.read() {
                Some(Some(orders)) if !orders.is_empty() => rsx! {
                    h2 { "Delivery History" }
                    div { class: "restaurant-list",
                        for order in orders.iter() {
                            {
                                let order_id = order["id"].as_str().unwrap_or("").to_string();
                                let status = order["status"].as_str().unwrap_or("Unknown");
                                let address = order["customer_address"].as_str().unwrap_or("—");
                                let food_total = order["pricing"]["food_total"].as_str().unwrap_or("0");
                                let is_active = status == "InDelivery";
                                rsx! {
                                    div {
                                        class: if is_active { "restaurant-card active-delivery" } else { "restaurant-card" },
                                        h3 {
                                            "Order #{order_id}",
                                            " ",
                                            span { class: "badge", "{status}" }
                                        }
                                        p { "Deliver to: {address}" }
                                        p { "Food total: ${food_total}" }
                                        if is_active {
                                            {
                                                let oid = order_id.clone();
                                                rsx! {
                                                    button {
                                                        class: "cta",
                                                        onclick: move |_| {
                                                            let oid = oid.clone();
                                                            spawn(async move {
                                                                let _ = api_patch_json(
                                                                    &format!("/orders/{oid}/status"),
                                                                    &serde_json::json!({"status": "Delivered"}),
                                                                ).await;
                                                                // Reload page
                                                                if let Some(w) = web_sys::window() {
                                                                    let _ = w.location().reload();
                                                                }
                                                            });
                                                        },
                                                        "Mark Delivered"
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
                Some(Some(_)) => rsx! {
                    p { "No deliveries yet." }
                },
                Some(None) => rsx! {
                    p { "Not registered as a courier." }
                    Link { to: Route::RegisterCourier {}, class: "cta", "Register Now" }
                },
                None => rsx! { p { "Loading deliveries..." } },
            }
        }
    }
}
