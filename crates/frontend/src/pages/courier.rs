#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::api::{api_get, api_patch_json, register_courier};
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

    let zones = use_resource(|| async {
        let restaurants: Vec<serde_json::Value> = api_get("/restaurants").await.ok()?;
        let mut seen = std::collections::HashMap::new();
        for r in &restaurants {
            if let Some(zid) = r["zone_id"].as_str() {
                seen.entry(zid.to_string())
                    .or_insert_with(|| zid.to_string());
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
                        if jwt.is_none() {
                            error.set(Some("Please sign in first".into()));
                            submitting.set(false);
                            return;
                        }
                        let body = serde_json::json!({
                            "name": n,
                            "zone_id": z,
                        });
                        match register_courier(&body).await {
                            Ok(_) => {
                                nav.push(Route::MyDeliveries {});
                            }
                            Err(e) => {
                                error.set(Some(e));
                            }
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
    let mut refresh = use_signal(|| 0u32);

    let courier = use_resource(move || {
        let jwt = user_state.read().jwt.clone();
        async move {
            let _jwt = jwt?;
            api_get::<serde_json::Value>("/couriers/me").await.ok()
        }
    });

    let deliveries = use_resource(move || {
        let _ = refresh();
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
                                                                refresh += 1;
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
