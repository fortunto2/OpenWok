#![allow(non_snake_case)]

use dioxus::prelude::*;

#[component]
pub fn RestaurantList() -> Element {
    rsx! { div { class: "page", h1 { "Restaurants" } p { "Loading..." } } }
}

#[component]
pub fn RestaurantMenu(id: String) -> Element {
    rsx! { div { class: "page", h1 { "Restaurant {id}" } } }
}
