#![allow(non_snake_case)]

use dioxus::prelude::*;

#[cfg(feature = "server")]
mod db;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        h1 { "OpenWok" }
        p { "Fair food delivery — $1 federal fee" }
    }
}
