#![allow(non_snake_case)]

mod analytics;
mod api;
mod app;
mod pages;
mod platform;
mod state;

fn main() {
    dioxus::launch(app::App);
}
