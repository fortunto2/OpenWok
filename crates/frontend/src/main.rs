#![allow(non_snake_case)]

use dioxus::prelude::*;
use openwok_core::money::Money;
use openwok_core::pricing::calculate_pricing;
use openwok_core::types::Restaurant;
use rust_decimal::Decimal;

// --- Cart state ---

#[derive(Clone, PartialEq)]
struct CartItem {
    menu_item_id: String,
    name: String,
    price: Money,
    quantity: u32,
}

#[derive(Clone, Default, PartialEq)]
struct CartState {
    items: Vec<CartItem>,
    restaurant_id: String,
    restaurant_name: String,
    zone_id: String,
}

// --- Routes ---

#[derive(Clone, Debug, PartialEq, Routable)]
#[rustfmt::skip]
enum Route {
    #[layout(Layout)]
        #[route("/")]
        Home {},
        #[route("/restaurants")]
        RestaurantList {},
        #[route("/restaurant/:id")]
        RestaurantMenu { id: String },
        #[route("/checkout")]
        Checkout {},
        #[route("/order/:id")]
        OrderTracking { id: String },
        #[route("/operator")]
        OperatorConsole {},
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(CartState::default()));
    rsx! { Router::<Route> {} }
}

#[component]
fn Layout() -> Element {
    rsx! {
        header { class: "header",
            nav { class: "nav",
                Link { to: Route::Home {}, class: "logo", "OpenWok" }
                div { class: "nav-links",
                    Link { to: Route::RestaurantList {}, "Restaurants" }
                    Link { to: Route::OperatorConsole {}, "Operator" }
                }
            }
        }
        main { class: "content",
            Outlet::<Route> {}
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        div { class: "hero",
            h1 { "OpenWok" }
            p { class: "subtitle", "Fair food delivery. $1 fee. Open-book pricing." }
            Link { to: Route::RestaurantList {}, class: "cta", "Browse Restaurants" }
        }
    }
}

// --- Server functions ---

const API_BASE: &str = "http://localhost:3000";

#[server]
async fn fetch_restaurants() -> Result<Vec<Restaurant>, ServerFnError> {
    let resp = reqwest::get(format!("{API_BASE}/restaurants")).await?;
    let data: Vec<Restaurant> = resp.json().await?;
    Ok(data)
}

#[server]
async fn fetch_restaurant(id: String) -> Result<Restaurant, ServerFnError> {
    let resp = reqwest::get(format!("{API_BASE}/restaurants/{id}")).await?;
    if !resp.status().is_success() {
        return Err(ServerFnError::ServerError("Restaurant not found".into()));
    }
    Ok(resp.json().await?)
}

#[server]
async fn place_order(body: String) -> Result<String, ServerFnError> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{API_BASE}/orders"))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?;
    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(ServerFnError::ServerError(msg));
    }
    let order: serde_json::Value = resp.json().await?;
    Ok(order["id"].as_str().unwrap_or_default().to_string())
}

// --- Helpers ---

fn cart_total(items: &[CartItem]) -> Money {
    items
        .iter()
        .map(|i| i.price * Decimal::from(i.quantity))
        .fold(Money::zero(), |a, b| a + b)
}

// --- Restaurant pages ---

#[component]
fn RestaurantList() -> Element {
    let restaurants = use_resource(fetch_restaurants);

    rsx! {
        h1 { "Restaurants" }
        match &*restaurants.read_unchecked() {
            Some(Ok(list)) if list.is_empty() => rsx! {
                p { "No restaurants available yet." }
            },
            Some(Ok(list)) => rsx! {
                div { class: "restaurant-grid",
                    for r in list {
                        RestaurantCard { key: "{r.id}", restaurant: r.clone() }
                    }
                }
            },
            Some(Err(e)) => rsx! {
                p { class: "error", "Failed to load restaurants: {e}" }
            },
            None => rsx! {
                p { "Loading restaurants..." }
            },
        }
    }
}

#[component]
fn RestaurantCard(restaurant: Restaurant) -> Element {
    rsx! {
        Link {
            to: Route::RestaurantMenu { id: restaurant.id.to_string() },
            class: "restaurant-card",
            h3 { "{restaurant.name}" }
            p { class: "item-count", "{restaurant.menu.len()} items" }
            div { class: "menu-preview",
                for item in restaurant.menu.iter().take(3) {
                    span { class: "menu-item", "{item.name} {item.price}" }
                }
            }
        }
    }
}

#[component]
fn RestaurantMenu(id: String) -> Element {
    let restaurant = use_resource(move || {
        let id = id.clone();
        async move { fetch_restaurant(id).await }
    });
    let mut cart = use_context::<Signal<CartState>>();

    match &*restaurant.read_unchecked() {
        Some(Ok(r)) => {
            let r = r.clone();
            rsx! {
                div { class: "menu-page",
                    div { class: "menu-section",
                        h1 { "{r.name}" }
                        div { class: "menu-items",
                            for item in r.menu.iter() {
                                div { class: "menu-item-row",
                                    div { class: "menu-item-info",
                                        span { class: "menu-item-name", "{item.name}" }
                                        span { class: "menu-item-price", "{item.price}" }
                                    }
                                    button {
                                        class: "add-btn",
                                        onclick: {
                                            let menu_item_id = item.id.to_string();
                                            let name = item.name.clone();
                                            let price = item.price;
                                            let rest_id = r.id.to_string();
                                            let rest_name = r.name.clone();
                                            let zone = r.zone_id.to_string();
                                            move |_| {
                                                let mut state = cart.write();
                                                state.restaurant_id = rest_id.clone();
                                                state.restaurant_name = rest_name.clone();
                                                state.zone_id = zone.clone();
                                                if let Some(existing) = state.items.iter_mut().find(|c| c.menu_item_id == menu_item_id) {
                                                    existing.quantity += 1;
                                                } else {
                                                    state.items.push(CartItem {
                                                        menu_item_id: menu_item_id.clone(),
                                                        name: name.clone(),
                                                        price,
                                                        quantity: 1,
                                                    });
                                                }
                                            }
                                        },
                                        "Add"
                                    }
                                }
                            }
                        }
                    }
                    CartPanel {}
                }
            }
        }
        Some(Err(e)) => rsx! { p { class: "error", "Error: {e}" } },
        None => rsx! { p { "Loading menu..." } },
    }
}

#[component]
fn CartPanel() -> Element {
    let cart = use_context::<Signal<CartState>>();
    let state = cart.read();

    if state.items.is_empty() {
        return rsx! {
            div { class: "cart-panel",
                h2 { "Cart" }
                p { "Your cart is empty" }
            }
        };
    }

    let total = cart_total(&state.items);
    let items: Vec<(String, u32, Money)> = state
        .items
        .iter()
        .map(|i| {
            let line = i.price * Decimal::from(i.quantity);
            (i.name.clone(), i.quantity, line)
        })
        .collect();

    rsx! {
        div { class: "cart-panel",
            h2 { "Cart" }
            for (name, qty, line_total) in items {
                div { class: "cart-item",
                    span { "{name} x{qty}" }
                    span { "{line_total}" }
                }
            }
            div { class: "cart-total",
                strong { "Total: {total}" }
            }
            Link { to: Route::Checkout {}, class: "checkout-btn", "Proceed to Order" }
        }
    }
}

// --- Checkout ---

#[component]
fn Checkout() -> Element {
    let mut cart = use_context::<Signal<CartState>>();
    let mut address = use_signal(String::new);
    let mut tip_input = use_signal(|| "3.00".to_string());
    let mut order_error = use_signal(|| None::<String>);
    let mut placing = use_signal(|| false);
    let nav = use_navigator();

    let state = cart.read();

    if state.items.is_empty() {
        return rsx! {
            h1 { "Checkout" }
            p { "Your cart is empty. " }
            Link { to: Route::RestaurantList {}, "Browse restaurants" }
        };
    }

    let food_total = cart_total(&state.items);
    let delivery_fee = Money::from("5.00");
    let local_ops_fee = Money::from("2.50");
    let tip = Money::from(tip_input.read().as_str());
    let pricing = calculate_pricing(food_total, delivery_fee, tip, local_ops_fee);
    let grand_total = pricing.total();

    let items_display: Vec<(String, u32, Money)> = state
        .items
        .iter()
        .map(|i| {
            let line = i.price * Decimal::from(i.quantity);
            (i.name.clone(), i.quantity, line)
        })
        .collect();

    let restaurant_name = state.restaurant_name.clone();
    let restaurant_id = state.restaurant_id.clone();
    let zone_id = state.zone_id.clone();

    let order_items: Vec<serde_json::Value> = state
        .items
        .iter()
        .map(|i| {
            serde_json::json!({
                "menu_item_id": i.menu_item_id,
                "name": i.name,
                "quantity": i.quantity,
                "unit_price": i.price.amount().to_string(),
            })
        })
        .collect();

    rsx! {
        div { class: "checkout-page",
            h1 { "Checkout" }
            h2 { "Order from {restaurant_name}" }

            // Cart summary
            div { class: "checkout-items",
                for (name, qty, line_total) in items_display {
                    div { class: "checkout-item",
                        span { "{name} x{qty}" }
                        span { "{line_total}" }
                    }
                }
            }

            // Delivery address
            div { class: "form-group",
                label { "Delivery Address" }
                input {
                    r#type: "text",
                    placeholder: "123 Main St, Los Angeles, CA",
                    value: "{address}",
                    oninput: move |e| address.set(e.value()),
                }
            }

            // Tip
            div { class: "form-group",
                label { "Tip ($)" }
                input {
                    r#type: "text",
                    value: "{tip_input}",
                    oninput: move |e| tip_input.set(e.value()),
                }
            }

            // 6-line pricing breakdown
            div { class: "pricing-breakdown",
                h3 { "Open-Book Receipt" }
                div { class: "price-line",
                    span { "Food Total" }
                    span { "{pricing.food_total}" }
                }
                div { class: "price-line",
                    span { "Delivery Fee" }
                    span { "{pricing.delivery_fee}" }
                }
                div { class: "price-line",
                    span { "Tip" }
                    span { "{pricing.tip}" }
                }
                div { class: "price-line",
                    span { "Federal Fee" }
                    span { "{pricing.federal_fee}" }
                }
                div { class: "price-line",
                    span { "Local Ops Fee" }
                    span { "{pricing.local_ops_fee}" }
                }
                div { class: "price-line",
                    span { "Processing (Stripe)" }
                    span { "{pricing.processing_fee}" }
                }
                div { class: "price-line total",
                    strong { "Total" }
                    strong { "{grand_total}" }
                }
            }

            // Error
            if let Some(err) = &*order_error.read() {
                p { class: "error", "{err}" }
            }

            // Place Order button
            button {
                class: "place-order-btn",
                disabled: placing() || address.read().is_empty(),
                onclick: move |_| {
                    let addr = address.read().clone();
                    let rid = restaurant_id.clone();
                    let zid = zone_id.clone();
                    let items_json = order_items.clone();
                    let tip_val = tip_input.read().clone();
                    async move {
                        placing.set(true);
                        let body = serde_json::json!({
                            "restaurant_id": rid,
                            "items": items_json,
                            "customer_address": addr,
                            "zone_id": zid,
                            "delivery_fee": "5.00",
                            "tip": tip_val,
                            "local_ops_fee": "2.50",
                        });
                        match place_order(body.to_string()).await {
                            Ok(order_id) => {
                                cart.write().items.clear();
                                nav.push(Route::OrderTracking { id: order_id });
                            }
                            Err(e) => {
                                order_error.set(Some(e.to_string()));
                                placing.set(false);
                            }
                        }
                    }
                },
                if placing() { "Placing order..." } else { "Place Order" }
            }
        }
    }
}

// --- Order Tracking ---

#[component]
fn OrderTracking(id: String) -> Element {
    rsx! {
        h1 { "Order Tracking" }
        p { "Order: {id}" }
    }
}

// --- Operator Console ---

#[component]
fn OperatorConsole() -> Element {
    rsx! {
        h1 { "Node Operator Console" }
        p { "Dashboard coming soon..." }
    }
}

fn main() {
    dioxus::launch(App);
}
