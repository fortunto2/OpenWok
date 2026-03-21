#![allow(non_snake_case)]

mod analytics;
mod api;
mod app;
mod local_db;
mod pages;
mod platform;
mod state;
mod sync;

fn main() {
    dioxus::launch(app::App);
}
