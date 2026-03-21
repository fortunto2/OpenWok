#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::app::Route;
use crate::state::UserState;

#[component]
pub fn RegisterCourier() -> Element {
    let user_state = use_context::<Signal<UserState>>();
    let nav = use_navigator();
    let mut name = use_signal(String::new);
    let mut zone_id = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    let zones = use_resource(|| async move {
        crate::server_fns::restaurants::get_restaurants()
            .await
            .ok()
            .map(|restaurants| {
                let mut seen = std::collections::BTreeMap::new();
                for restaurant in restaurants {
                    seen.entry(restaurant.zone_id.to_string())
                        .or_insert_with(|| restaurant.zone_id.to_string());
                }
                seen.into_iter().collect::<Vec<(String, String)>>()
            })
    });

    rsx! {
        div { class: "page onboard",
            h1 { "Register as Courier" }
            p { "Join OpenWok's delivery network. Pick your zone and start delivering." }
            form {
                class: "onboard-form",
                onsubmit: move |event| {
                    event.stop_propagation();
                    let token = user_state.read().jwt.clone();
                    let name = name.read().clone();
                    let zone_id = zone_id.read().clone();
                    let nav = nav;
                    submitting.set(true);
                    error.set(None);
                    spawn(async move {
                        let Some(token) = token else {
                            error.set(Some("Please sign in first".into()));
                            submitting.set(false);
                            return;
                        };
                        match crate::server_fns::couriers::register_courier(token, name, zone_id).await {
                            Ok(_) => {
                                let _ = nav.push(Route::MyDeliveries {});
                            }
                            Err(err) => {
                                error.set(Some(err.to_string()));
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
                        oninput: move |event| name.set(event.value()),
                    }
                }
                div { class: "form-group",
                    label { "Delivery Zone *" }
                    match &*zones.read_unchecked() {
                        Some(Some(zone_list)) if !zone_list.is_empty() => rsx! {
                            select {
                                required: true,
                                value: "{zone_id}",
                                onchange: move |event| zone_id.set(event.value()),
                                option { value: "", "Select a zone..." }
                                for (zone_id, label) in zone_list.iter() {
                                    option { value: "{zone_id}", "{label}" }
                                }
                            }
                        },
                        _ => rsx! {
                            input {
                                r#type: "text",
                                required: true,
                                placeholder: "Zone UUID",
                                value: "{zone_id}",
                                oninput: move |event| zone_id.set(event.value()),
                            }
                        },
                    }
                }
                if let Some(err) = &*error.read() {
                    p { class: "error", "{err}" }
                }
                button {
                    r#type: "submit",
                    class: "cta",
                    disabled: submitting(),
                    if submitting() { "Registering..." } else { "Register as Courier" }
                }
            }
        }
    }
}

#[component]
pub fn MyDeliveries() -> Element {
    let user_state = use_context::<Signal<UserState>>();
    let refresh = use_signal(|| 0u32);
    let token = user_state.read().jwt.clone();
    let courier_token = token.clone();
    let deliveries_token = token.clone();

    let courier = use_resource(move || {
        let token = courier_token.clone();
        async move {
            let token = token?;
            crate::server_fns::couriers::get_courier_me(token)
                .await
                .ok()
        }
    });

    let deliveries = use_resource(move || {
        let _ = refresh();
        let token = deliveries_token.clone();
        async move {
            let token = token?;
            crate::server_fns::couriers::get_my_deliveries(token)
                .await
                .ok()
        }
    });

    let courier_state = courier.read_unchecked().as_ref().cloned();
    let deliveries_state = deliveries.read_unchecked().as_ref().cloned();

    rsx! {
        div { class: "page my-restaurants",
            h1 { "My Deliveries" }

            match courier_state {
                Some(Some(courier)) => rsx! {
                    div { class: "courier-profile",
                        p { "Courier: {courier.name}" }
                        p {
                            "Status: "
                            if courier.available {
                                span { class: "badge active", "Available" }
                            } else {
                                span { class: "badge", "Busy" }
                            }
                        }
                        button {
                            class: "action-btn",
                            onclick: move |_| {
                                let token = user_state.read().jwt.clone();
                                let available = !courier.available;
                                let mut refresh = refresh;
                                spawn(async move {
                                    if let Some(token) = token {
                                        let _ = crate::server_fns::couriers::set_my_availability(token, available).await;
                                        refresh += 1;
                                    }
                                });
                            },
                            if courier.available { "Set Busy" } else { "Set Available" }
                        }
                    }
                },
                Some(None) => rsx! {
                    p { "Not registered as a courier." }
                    Link { to: Route::RegisterCourier {}, class: "cta", "Register Now" }
                },
                None => rsx! {
                    p { "Loading..." }
                },
            }

            match deliveries_state {
                Some(Some(orders)) if !orders.is_empty() => rsx! {
                    h2 { "Delivery History" }
                    div { class: "restaurant-list",
                        for order in orders.iter() {
                            {
                                let order_id = order.id.to_string();
                                let status = format!("{:?}", order.status);
                                let is_active = status == "InDelivery";
                                let food_total = order.pricing.food_total.to_string();
                                rsx! {
                                    div {
                                        class: if is_active { "restaurant-card active-delivery" } else { "restaurant-card" },
                                        key: "{order.id}",
                                        h3 {
                                            "Order #{order.id} "
                                            span { class: "badge", "{status}" }
                                        }
                                        p { "Deliver to: {order.customer_address}" }
                                        p { "Food total: {food_total}" }
                                        if is_active {
                                            button {
                                                class: "cta",
                                                onclick: move |_| {
                                                    let token = user_state.read().jwt.clone();
                                                    let order_id = order_id.clone();
                                                    let mut refresh = refresh;
                                                    spawn(async move {
                                                        if let Some(token) = token {
                                                            let _ = crate::server_fns::couriers::mark_delivery_completed(token, order_id).await;
                                                            refresh += 1;
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
                },
                Some(Some(_)) => rsx! {
                    p { "No deliveries yet." }
                },
                Some(None) => rsx! {
                    p { "Not registered as a courier." }
                    Link { to: Route::RegisterCourier {}, class: "cta", "Register Now" }
                },
                None => rsx! {
                    p { "Loading deliveries..." }
                },
            }
        }
    }
}
