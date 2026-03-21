#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::app::Route;

#[component]
pub fn Home() -> Element {
    rsx! {
        div { class: "hero",
            h1 { "OpenWok" }
            p { class: "subtitle", "Fair food delivery. $1 fee. Open-book pricing." }
            Link { to: Route::RestaurantList {}, class: "cta", "Browse Restaurants" }
        }
    }
}
