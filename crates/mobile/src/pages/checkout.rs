#![allow(non_snake_case)]

use dioxus::prelude::*;

#[component]
pub fn Checkout() -> Element {
    rsx! { div { class: "page", h1 { "Checkout" } } }
}
