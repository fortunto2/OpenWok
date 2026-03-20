#![allow(non_snake_case)]

use dioxus::prelude::*;

#[component]
pub fn OrderTracking(id: String) -> Element {
    rsx! { div { class: "page", h1 { "Order #{id}" } } }
}

#[component]
pub fn OrderSuccess(id: String) -> Element {
    rsx! { div { class: "page", h1 { "Order Confirmed!" } p { "Order #{id}" } } }
}
