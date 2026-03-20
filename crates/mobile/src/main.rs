#![allow(non_snake_case)]
#![allow(dead_code)]

mod api;
mod app;
mod auth;
mod config;
mod pages;
mod state;
mod storage;

fn main() {
    dioxus::launch(app::App);
}
