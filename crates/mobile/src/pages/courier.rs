#![allow(non_snake_case)]

use dioxus::prelude::*;

#[component]
pub fn RegisterCourier() -> Element {
    rsx! { div { class: "page", h1 { "Register as Courier" } } }
}

#[component]
pub fn MyDeliveries() -> Element {
    rsx! { div { class: "page", h1 { "My Deliveries" } } }
}
