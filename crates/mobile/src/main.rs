#![allow(non_snake_case)]

use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    rsx! {
        div {
            style: "display:flex;align-items:center;justify-content:center;height:100vh;font-family:system-ui;",
            h1 { "OpenWok" }
        }
    }
}
