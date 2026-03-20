#![allow(non_snake_case)]

use dioxus::prelude::*;
use gloo_net::http::Request;
use openwok_core::money::Money;
use openwok_core::pricing::calculate_pricing;
use openwok_core::types::Restaurant;
use rust_decimal::Decimal;
use wasm_bindgen::prelude::*;

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

// --- PostHog Analytics ---

fn posthog_capture(event: &str) {
    let window = web_sys::window().unwrap();
    if let Ok(ph) = js_sys::Reflect::get(&window, &JsValue::from_str("posthog")) {
        if !ph.is_undefined() && !ph.is_null() {
            if let Ok(capture_fn) = js_sys::Reflect::get(&ph, &JsValue::from_str("capture")) {
                if let Some(func) = capture_fn.dyn_ref::<js_sys::Function>() {
                    let _ = func.call1(&ph, &JsValue::from_str(event));
                }
            }
        }
    }
}

fn posthog_capture_with_props(event: &str, props: &serde_json::Value) {
    let window = web_sys::window().unwrap();
    if let Ok(ph) = js_sys::Reflect::get(&window, &JsValue::from_str("posthog")) {
        if !ph.is_undefined() && !ph.is_null() {
            if let Ok(capture_fn) = js_sys::Reflect::get(&ph, &JsValue::from_str("capture")) {
                if let Some(func) = capture_fn.dyn_ref::<js_sys::Function>() {
                    let props_js =
                        js_sys::JSON::parse(&props.to_string()).unwrap_or(JsValue::NULL);
                    let _ = func.call2(&ph, &JsValue::from_str(event), &props_js);
                }
            }
        }
    }
}

const POSTHOG_SNIPPET: &str = r#"
!function(t,e){var o,n,p,r;e.__SV||(window.posthog=e,e._i=[],e.init=function(i,s,a){function g(t,e){var o=e.split(".");2==o.length&&(t=t[o[0]],e=o[1]),t[e]=function(){t.push([e].concat(Array.prototype.slice.call(arguments,0)))}}(p=t.createElement("script")).type="text/javascript",p.async=!0,p.src=s.api_host+"/static/array.js",(r=t.getElementsByTagName("script")[0]).parentNode.insertBefore(p,r);var u=e;for(void 0!==a?u=e[a]=[]:a="posthog",u.people=u.people||[],u.toString=function(t){var e="posthog";return"posthog"!==a&&(e+="."+a),t||(e+=" (stub)"),e},u.people.toString=function(){return u.toString(1)+".people (stub)"},o="capture identify alias people.set people.set_once set_config register register_once unregister opt_out_capturing has_opted_out_capturing opt_in_capturing reset isFeatureEnabled onFeatureFlags getFeatureFlag getFeatureFlagPayload reloadFeatureFlags group updateEarlyAccessFeatureEnrollment getEarlyAccessFeatures getActiveMatchingSurveys getSurveys onSessionId".split(" "),n=0;n<o.length;n++)g(u,o[n]);e._i.push([i,s,a])},e.__SV=1)}(document,window.posthog||[]);
if (window.__POSTHOG_API_KEY__) {
    posthog.init(window.__POSTHOG_API_KEY__, {api_host: 'https://eu.posthog.com'});
}
"#;

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
        #[route("/economics")]
        PublicEconomicsPage {},
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(CartState::default()));
    rsx! {
        document::Script { {POSTHOG_SNIPPET} }
        Router::<Route> {}
    }
}

#[component]
fn Layout() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/style.css") }
        header { class: "header",
            nav { class: "nav",
                Link { to: Route::Home {}, class: "logo", "OpenWok" }
                div { class: "nav-links",
                    Link { to: Route::RestaurantList {}, "Restaurants" }
                    Link { to: Route::PublicEconomicsPage {}, "Economics" }
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

// --- API helpers ---

const API_BASE: &str = "/api";

async fn api_get<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    let resp = Request::get(&format!("{API_BASE}{path}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

async fn api_post_json<T: serde::de::DeserializeOwned>(
    path: &str,
    body: &str,
) -> Result<T, String> {
    let resp = Request::post(&format!("{API_BASE}{path}"))
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(msg);
    }
    resp.json().await.map_err(|e| e.to_string())
}

async fn api_patch_json(path: &str, body: &serde_json::Value) -> Result<(), String> {
    let resp = Request::patch(&format!("{API_BASE}{path}"))
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(msg);
    }
    Ok(())
}

// --- Data fetchers ---

async fn fetch_restaurants() -> Result<Vec<Restaurant>, String> {
    api_get("/restaurants").await
}

async fn fetch_restaurant(id: String) -> Result<Restaurant, String> {
    api_get(&format!("/restaurants/{id}")).await
}

async fn place_order(body: String) -> Result<String, String> {
    let order: serde_json::Value = api_post_json("/orders", &body).await?;
    Ok(order["id"].as_str().unwrap_or_default().to_string())
}

async fn fetch_order(id: String) -> Result<serde_json::Value, String> {
    api_get(&format!("/orders/{id}")).await
}

async fn fetch_dashboard() -> Result<serde_json::Value, String> {
    let restaurants: Vec<serde_json::Value> = api_get("/restaurants").await?;
    let couriers: Vec<serde_json::Value> = api_get("/couriers").await?;
    Ok(serde_json::json!({
        "restaurant_count": restaurants.len(),
        "couriers_online": couriers.len(),
        "restaurants": restaurants,
        "couriers": couriers,
    }))
}

async fn fetch_economics() -> Result<serde_json::Value, String> {
    api_get("/public/economics").await
}

async fn fetch_admin_metrics() -> Result<serde_json::Value, String> {
    api_get("/admin/metrics").await
}

async fn fetch_all_orders() -> Result<Vec<serde_json::Value>, String> {
    api_get("/orders").await
}

async fn assign_courier(order_id: String) -> Result<serde_json::Value, String> {
    let resp = Request::post(&format!("{API_BASE}/orders/{order_id}/assign"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(msg);
    }
    resp.json().await.map_err(|e| e.to_string())
}

async fn transition_order(order_id: String, status: String) -> Result<(), String> {
    api_patch_json(
        &format!("/orders/{order_id}/status"),
        &serde_json::json!({ "status": status }),
    )
    .await
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
            posthog_capture("restaurant_view");
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
                                                posthog_capture("add_to_cart");
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

    posthog_capture("checkout_start");

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
                                posthog_capture("order_placed");
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

const ORDER_TIMELINE: &[&str] = &[
    "Created",
    "Confirmed",
    "Preparing",
    "ReadyForPickup",
    "InDelivery",
    "Delivered",
];

#[component]
fn OrderTracking(id: String) -> Element {
    let mut refresh = use_signal(|| 0u32);
    let order = use_resource(move || {
        let _ = refresh();
        let id = id.clone();
        async move { fetch_order(id).await }
    });

    match &*order.read_unchecked() {
        Some(Ok(data)) => {
            let status = data["status"].as_str().unwrap_or("Unknown").to_string();
            let pricing = &data["pricing"];

            let food_total = pricing["food_total"].as_str().unwrap_or("0").to_string();
            let delivery_fee = pricing["delivery_fee"].as_str().unwrap_or("0").to_string();
            let tip = pricing["tip"].as_str().unwrap_or("0").to_string();
            let federal_fee = pricing["federal_fee"].as_str().unwrap_or("0").to_string();
            let local_ops_fee = pricing["local_ops_fee"].as_str().unwrap_or("0").to_string();
            let processing_fee = pricing["processing_fee"]
                .as_str()
                .unwrap_or("0")
                .to_string();

            let items: Vec<(String, u64, String)> = data["items"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|i| {
                            (
                                i["name"].as_str().unwrap_or("").to_string(),
                                i["quantity"].as_u64().unwrap_or(1),
                                i["unit_price"].as_str().unwrap_or("0").to_string(),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();

            let current_idx = ORDER_TIMELINE
                .iter()
                .position(|s| *s == status)
                .unwrap_or(0);
            let is_terminal = status == "Delivered" || status == "Cancelled";

            let timeline: Vec<(&str, &str)> = ORDER_TIMELINE
                .iter()
                .enumerate()
                .map(|(idx, step)| {
                    let class = if idx < current_idx {
                        "step done"
                    } else if idx == current_idx {
                        "step current"
                    } else {
                        "step"
                    };
                    (class, *step)
                })
                .collect();

            rsx! {
                div { class: "order-tracking",
                    h1 { "Order Tracking" }

                    // Status timeline
                    div { class: "timeline",
                        for (class, step) in timeline {
                            div {
                                class: "{class}",
                                span { class: "step-dot" }
                                span { class: "step-label", "{step}" }
                            }
                        }
                    }

                    if !is_terminal {
                        button {
                            class: "refresh-btn",
                            onclick: move |_| refresh += 1,
                            "Refresh Status"
                        }
                    }

                    // Order items
                    div { class: "order-items",
                        h3 { "Items" }
                        for (name, qty, price) in items {
                            div { class: "order-item",
                                span { "{name} x{qty}" }
                                span { "${price}" }
                            }
                        }
                    }

                    // Pricing breakdown (always visible)
                    div { class: "pricing-breakdown",
                        h3 { "Open-Book Receipt" }
                        div { class: "price-line",
                            span { "Food Total" }
                            span { "${food_total}" }
                        }
                        div { class: "price-line",
                            span { "Delivery Fee" }
                            span { "${delivery_fee}" }
                        }
                        div { class: "price-line",
                            span { "Tip" }
                            span { "${tip}" }
                        }
                        div { class: "price-line",
                            span { "Federal Fee" }
                            span { "${federal_fee}" }
                        }
                        div { class: "price-line",
                            span { "Local Ops Fee" }
                            span { "${local_ops_fee}" }
                        }
                        div { class: "price-line",
                            span { "Processing (Stripe)" }
                            span { "${processing_fee}" }
                        }
                    }
                }
            }
        }
        Some(Err(e)) => rsx! {
            h1 { "Order Tracking" }
            p { class: "error", "Error: {e}" }
        },
        None => rsx! {
            h1 { "Order Tracking" }
            p { "Loading order..." }
        },
    }
}

// --- Public Economics Page ---

#[component]
fn PublicEconomicsPage() -> Element {
    let economics = use_resource(fetch_economics);

    rsx! {
        div { class: "economics-page",
            div { class: "hero",
                h1 { "Open-Book Economics" }
                p { class: "subtitle",
                    "Every dollar traced. No hidden fees. See exactly where your money goes."
                }
            }

            match &*economics.read_unchecked() {
                Some(Ok(data)) => {
                    let total_orders = data["total_orders"].as_i64().unwrap_or(0);
                    let food = data["total_food_revenue"].as_str().unwrap_or("0.00").to_string();
                    let delivery = data["total_delivery_fees"].as_str().unwrap_or("0.00").to_string();
                    let federal = data["total_federal_fees"].as_str().unwrap_or("0.00").to_string();
                    let local_ops = data["total_local_ops_fees"].as_str().unwrap_or("0.00").to_string();
                    let processing = data["total_processing_fees"].as_str().unwrap_or("0.00").to_string();
                    let avg = data["avg_order_value"].as_str().unwrap_or("0.00").to_string();

                    rsx! {
                        div { class: "stats-grid",
                            div { class: "stat-card",
                                h3 { "{total_orders}" }
                                p { "Total Orders" }
                            }
                            div { class: "stat-card",
                                h3 { "${avg}" }
                                p { "Avg Order Value" }
                            }
                        }

                        div { class: "pricing-breakdown",
                            h3 { "Where Your Money Goes" }
                            div { class: "price-line",
                                span { "Food Revenue (100% to restaurants)" }
                                span { "${food}" }
                            }
                            div { class: "price-line",
                                span { "Delivery Fees (100% to couriers)" }
                                span { "${delivery}" }
                            }
                            div { class: "price-line",
                                span { "Federal Fee ($1/order — protocol & security)" }
                                span { "${federal}" }
                            }
                            div { class: "price-line",
                                span { "Local Ops Fee (node operator costs)" }
                                span { "${local_ops}" }
                            }
                            div { class: "price-line",
                                span { "Processing (Stripe pass-through)" }
                                span { "${processing}" }
                            }
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

// --- Operator Console ---

#[component]
fn OperatorConsole() -> Element {
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
            let food_rev = revenue["total_food_revenue"].as_str().unwrap_or("0.00").to_string();
            let delivery_rev = revenue["total_delivery_fees"].as_str().unwrap_or("0.00").to_string();
            let federal_rev = revenue["total_federal_fees"].as_str().unwrap_or("0.00").to_string();
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
        },
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

fn main() {
    dioxus::launch(App);
}
