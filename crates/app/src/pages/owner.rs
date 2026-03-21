#![allow(non_snake_case)]

use dioxus::prelude::*;
use openwok_core::types::Restaurant;

use crate::app::Route;
use crate::state::UserState;

#[component]
pub fn MyRestaurants() -> Element {
    let user_state = use_context::<Signal<UserState>>();
    let token = user_state.read().jwt.clone();

    let restaurants: Resource<Vec<Restaurant>> = use_resource(move || {
        let token = token.clone();
        async move {
            let Some(token) = token else {
                return Vec::new();
            };
            crate::server_fns::owner::my_restaurants(token)
                .await
                .unwrap_or_default()
        }
    });

    rsx! {
        div { class: "page my-restaurants",
            div { class: "page-header",
                h1 { "My Restaurants" }
                Link { to: Route::OnboardRestaurant {}, class: "cta", "Add Restaurant" }
            }
            match &*restaurants.read_unchecked() {
                Some(list) if !list.is_empty() => rsx! {
                    div { class: "restaurant-grid",
                        for restaurant in list {
                            div { class: "restaurant-card", key: "{restaurant.id}",
                                h3 { "{restaurant.name}" }
                                p { class: if restaurant.active { "status active" } else { "status inactive" },
                                    if restaurant.active { "Active" } else { "Inactive" }
                                }
                                p { "{restaurant.menu.len()} menu items" }
                                if let Some(desc) = &restaurant.description {
                                    p { class: "desc", "{desc}" }
                                }
                                Link {
                                    to: Route::RestaurantSettings { id: restaurant.id.to_string() },
                                    class: "settings-link",
                                    "Manage ->"
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
    let nav = use_navigator();
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
                onsubmit: move |event| {
                    event.stop_propagation();
                    let token = user_state.read().jwt.clone();
                    let input = crate::server_fns::owner::CreateRestaurantInput {
                        name: name.read().clone(),
                        zone_id: zone_id.read().clone(),
                        description: optional_string(description.read().clone()),
                        address: optional_string(address.read().clone()),
                        phone: optional_string(phone.read().clone()),
                    };
                    let nav = nav;
                    submitting.set(true);
                    error.set(None);
                    spawn(async move {
                        let Some(token) = token else {
                            error.set(Some("Please sign in first".into()));
                            submitting.set(false);
                            return;
                        };
                        match crate::server_fns::owner::create_restaurant(token, input).await {
                            Ok(restaurant) => {
                                let _ = nav.push(Route::RestaurantSettings {
                                    id: restaurant.id.to_string(),
                                });
                            }
                            Err(err) => {
                                error.set(Some(err.to_string()));
                            }
                        }
                        submitting.set(false);
                    });
                },
                div { class: "form-group",
                    label { "Restaurant Name *" }
                    input { r#type: "text", required: true, value: "{name}", oninput: move |event| name.set(event.value()) }
                }
                div { class: "form-group",
                    label { "Zone ID *" }
                    input { r#type: "text", required: true, placeholder: "UUID of the zone", value: "{zone_id}", oninput: move |event| zone_id.set(event.value()) }
                }
                div { class: "form-group",
                    label { "Description" }
                    textarea { value: "{description}", oninput: move |event| description.set(event.value()) }
                }
                div { class: "form-group",
                    label { "Address" }
                    input { r#type: "text", value: "{address}", oninput: move |event| address.set(event.value()) }
                }
                div { class: "form-group",
                    label { "Phone" }
                    input { r#type: "tel", value: "{phone}", oninput: move |event| phone.set(event.value()) }
                }
                if let Some(err) = &*error.read() {
                    p { class: "error", "{err}" }
                }
                button {
                    r#type: "submit",
                    class: "cta",
                    disabled: submitting(),
                    if submitting() { "Creating..." } else { "Create Restaurant" }
                }
            }
        }
    }
}

#[component]
pub fn RestaurantSettings(id: String) -> Element {
    let user_state = use_context::<Signal<UserState>>();
    let token = user_state.read().jwt.clone();
    let mut refresh = use_signal(|| 0u32);
    let mut active_tab = use_signal(|| "info".to_string());
    let restaurant_route_id = id.clone();
    let restaurant_filter_id = id.clone();

    let restaurant = use_resource(move || {
        let _ = refresh();
        let id = restaurant_route_id.clone();
        async move {
            crate::server_fns::restaurants::get_restaurant(id)
                .await
                .ok()
        }
    });
    let owner_orders = use_resource(move || {
        let _ = refresh();
        let token = token.clone();
        async move {
            let Some(token) = token else {
                return Vec::new();
            };
            crate::server_fns::owner::my_orders(token)
                .await
                .unwrap_or_default()
        }
    });

    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut address = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut initialized = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut save_msg = use_signal(|| None::<String>);

    let mut new_item_name = use_signal(String::new);
    let mut new_item_price = use_signal(String::new);
    let mut adding_item = use_signal(|| false);

    let restaurant_resource_state = restaurant.read_unchecked().as_ref().cloned();
    let restaurant_loaded = restaurant_resource_state.is_some();
    let restaurant_state = restaurant_resource_state.flatten();
    let owner_order_list = owner_orders
        .read_unchecked()
        .as_ref()
        .cloned()
        .unwrap_or_default();

    if let Some(current_restaurant) = restaurant_state.as_ref()
        && !initialized()
    {
        name.set(current_restaurant.name.clone());
        description.set(current_restaurant.description.clone().unwrap_or_default());
        address.set(current_restaurant.address.clone().unwrap_or_default());
        phone.set(current_restaurant.phone.clone().unwrap_or_default());
        initialized.set(true);
    }

    rsx! {
        div { class: "page restaurant-settings",
            match restaurant_state {
                Some(restaurant) => {
                    let orders: Vec<_> = owner_order_list
                        .iter()
                        .filter(|order| order.restaurant_id.to_string() == restaurant_filter_id)
                        .cloned()
                        .collect();
                    let save_restaurant_id = restaurant_filter_id.clone();
                    let toggle_restaurant_id = restaurant_filter_id.clone();
                    let add_item_restaurant_id = restaurant_filter_id.clone();

                    rsx! {
                        h1 { "Settings: {restaurant.name}" }
                        div { class: "tab-nav",
                            button {
                                class: if *active_tab.read() == "info" { "tab active" } else { "tab" },
                                onclick: move |_| active_tab.set("info".into()),
                                "Info"
                            }
                            button {
                                class: if *active_tab.read() == "menu" { "tab active" } else { "tab" },
                                onclick: move |_| active_tab.set("menu".into()),
                                "Menu"
                            }
                            button {
                                class: if *active_tab.read() == "orders" { "tab active" } else { "tab" },
                                onclick: move |_| active_tab.set("orders".into()),
                                "Orders"
                            }
                        }

                        if *active_tab.read() == "info" {
                            div { class: "settings-section",
                                h2 { "Restaurant Info" }
                                div { class: "form-group", label { "Name" } input { r#type: "text", value: "{name}", oninput: move |event| name.set(event.value()) } }
                                div { class: "form-group", label { "Description" } textarea { value: "{description}", oninput: move |event| description.set(event.value()) } }
                                div { class: "form-group", label { "Address" } input { r#type: "text", value: "{address}", oninput: move |event| address.set(event.value()) } }
                                div { class: "form-group", label { "Phone" } input { r#type: "tel", value: "{phone}", oninput: move |event| phone.set(event.value()) } }
                                button {
                                    class: "cta",
                                    disabled: saving(),
                                    onclick: move |_| {
                                        let token = user_state.read().jwt.clone();
                                        let restaurant_id = save_restaurant_id.clone();
                                        let input = crate::server_fns::owner::UpdateRestaurantInput {
                                            name: Some(name.read().clone()),
                                            description: optional_string(description.read().clone()),
                                            address: optional_string(address.read().clone()),
                                            phone: optional_string(phone.read().clone()),
                                        };
                                        saving.set(true);
                                        save_msg.set(None);
                                        spawn(async move {
                                            let Some(token) = token else {
                                                save_msg.set(Some("Please sign in first".into()));
                                                saving.set(false);
                                                return;
                                            };
                                            match crate::server_fns::owner::update_restaurant(token, restaurant_id, input).await {
                                                Ok(_) => {
                                                    save_msg.set(Some("Saved!".into()));
                                                    refresh += 1;
                                                }
                                                Err(_) => save_msg.set(Some("Failed to save".into())),
                                            }
                                            saving.set(false);
                                        });
                                    },
                                    if saving() { "Saving..." } else { "Save Changes" }
                                }
                                if let Some(msg) = &*save_msg.read() {
                                    p { class: "save-msg", "{msg}" }
                                }
                                div { class: "status-section",
                                    h3 { "Status" }
                                    p { "Currently: " strong { if restaurant.active { "Active" } else { "Inactive" } } }
                                    button {
                                        class: if restaurant.active { "btn-danger" } else { "cta" },
                                        onclick: move |_| {
                                            let token = user_state.read().jwt.clone();
                                            let restaurant_id = toggle_restaurant_id.clone();
                                            let active = !restaurant.active;
                                            spawn(async move {
                                                if let Some(token) = token {
                                                    let _ = crate::server_fns::owner::toggle_restaurant_active(token, restaurant_id, active).await;
                                                    refresh += 1;
                                                }
                                            });
                                        },
                                        if restaurant.active { "Deactivate" } else { "Activate" }
                                    }
                                }
                            }
                        }

                        if *active_tab.read() == "menu" {
                            div { class: "settings-section",
                                h2 { "Menu ({restaurant.menu.len()} items)" }
                                for item in restaurant.menu.iter() {
                                    {
                                        let item_id = item.id.to_string();
                                        let item_name = item.name.clone();
                                        let item_price = item.price.to_string();
                                        rsx! {
                                            div { class: "menu-item-row", key: "{item_id}",
                                                span { class: "item-name", "{item_name}" }
                                                span { class: "item-price", "{item_price}" }
                                                button {
                                                    class: "btn-small btn-danger",
                                                    onclick: move |_| {
                                                        let token = user_state.read().jwt.clone();
                                                        let item_id = item_id.clone();
                                                        spawn(async move {
                                                            if let Some(token) = token {
                                                                let _ = crate::server_fns::owner::delete_menu_item(token, item_id).await;
                                                                refresh += 1;
                                                            }
                                                        });
                                                    },
                                                    "Delete"
                                                }
                                            }
                                        }
                                    }
                                }
                                div { class: "add-item-form",
                                    h3 { "Add Menu Item" }
                                    input { r#type: "text", placeholder: "Item name", value: "{new_item_name}", oninput: move |event| new_item_name.set(event.value()) }
                                    input { r#type: "text", placeholder: "Price (e.g. 12.99)", value: "{new_item_price}", oninput: move |event| new_item_price.set(event.value()) }
                                    button {
                                        class: "cta",
                                        disabled: adding_item(),
                                        onclick: move |_| {
                                            let token = user_state.read().jwt.clone();
                                            let restaurant_id = add_item_restaurant_id.clone();
                                            let name = new_item_name.read().clone();
                                            let price = new_item_price.read().clone();
                                            if name.is_empty() || price.is_empty() {
                                                return;
                                            }
                                            adding_item.set(true);
                                            spawn(async move {
                                                if let Some(token) = token {
                                                    let _ = crate::server_fns::owner::add_menu_item(token, restaurant_id, name, price).await;
                                                    refresh += 1;
                                                }
                                                adding_item.set(false);
                                            });
                                        },
                                        if adding_item() { "Adding..." } else { "Add Item" }
                                    }
                                }
                            }
                        }

                        if *active_tab.read() == "orders" {
                            div { class: "settings-section",
                                h2 { "Orders" }
                                if orders.is_empty() {
                                    p { "No orders yet." }
                                } else {
                                    div { class: "restaurant-list",
                                        for order in orders.iter() {
                                            {
                                                let order_id = order.id.to_string();
                                                let status = format!("{:?}", order.status);
                                                let total = order.pricing.total().to_string();
                                                let address = order.customer_address.clone();
                                                let created_at = order.created_at.to_string();
                                                let items = order
                                                    .items
                                                    .iter()
                                                    .map(|item| format!("{}x {}", item.quantity, item.name))
                                                    .collect::<Vec<_>>();
                                                rsx! {
                                                    OwnerOrderCard {
                                                        key: "{order_id}",
                                                        order_id: order_id.clone(),
                                                        status: status,
                                                        total: total,
                                                        address: address,
                                                        created_at: created_at,
                                                        items: items,
                                                        refresh: refresh,
                                                        token: user_state.read().jwt.clone(),
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
                None if restaurant_loaded => rsx! { p { "Restaurant not found" } },
                None => rsx! { p { "Loading..." } },
            }
        }
    }
}

#[component]
fn OwnerOrderCard(
    order_id: String,
    status: String,
    total: String,
    address: String,
    created_at: String,
    items: Vec<String>,
    refresh: Signal<u32>,
    token: Option<String>,
) -> Element {
    let status_class = match status.as_str() {
        "Confirmed" => "badge confirmed",
        "Preparing" => "badge preparing",
        "ReadyForPickup" => "badge ready",
        "InDelivery" => "badge active",
        "Delivered" => "badge delivered",
        "Cancelled" => "badge cancelled",
        _ => "badge",
    };

    rsx! {
        div { class: "restaurant-card order-card",
            h3 { "#{order_id}" " " span { class: "{status_class}", "{status}" } }
            ul { class: "order-items",
                for item in items.iter() {
                    li { "{item}" }
                }
            }
            p { "Total: {total}" }
            p { class: "order-address", "Deliver to: {address}" }
            p { class: "order-time", "{created_at}" }
            div { class: "order-actions",
                if status == "Confirmed" {
                    OwnerOrderActionButton { token: token.clone(), order_id: order_id.clone(), next_status: "Preparing", label: "Accept", refresh: refresh }
                    OwnerOrderActionButton { token: token.clone(), order_id: order_id.clone(), next_status: "Cancelled", label: "Cancel", refresh: refresh }
                }
                if status == "Preparing" {
                    OwnerOrderActionButton { token: token.clone(), order_id: order_id.clone(), next_status: "ReadyForPickup", label: "Mark Ready", refresh: refresh }
                    OwnerOrderActionButton { token: token.clone(), order_id: order_id.clone(), next_status: "Cancelled", label: "Cancel", refresh: refresh }
                }
            }
        }
    }
}

#[component]
fn OwnerOrderActionButton(
    token: Option<String>,
    order_id: String,
    next_status: &'static str,
    label: &'static str,
    refresh: Signal<u32>,
) -> Element {
    rsx! {
        button {
            class: if next_status == "Cancelled" { "btn-danger" } else { "cta" },
            onclick: move |_| {
                let token = token.clone();
                let order_id = order_id.clone();
                spawn(async move {
                    if let Some(token) = token {
                        let _ = crate::server_fns::owner::update_owned_order_status(
                            token,
                            order_id,
                            next_status.to_string(),
                        )
                        .await;
                        refresh += 1;
                    }
                });
            },
            "{label}"
        }
    }
}

fn optional_string(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}
