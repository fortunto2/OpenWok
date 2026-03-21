#![allow(non_snake_case)]

/// Open a URL in the system browser (or navigate on web).
#[cfg(target_arch = "wasm32")]
pub fn open_url(url: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(url);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn open_url(url: &str) {
    let _ = open::that(url);
}

/// Reload the current page (web) or no-op (native — use signal refresh instead).
#[cfg(target_arch = "wasm32")]
pub fn reload_page() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().reload();
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn reload_page() {
    // No-op on native — use Dioxus signal-based refresh
}

/// Check if device is online.
#[cfg(target_arch = "wasm32")]
pub fn is_online() -> bool {
    web_sys::window()
        .map(|w| w.navigator().on_line())
        .unwrap_or(true)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn is_online() -> bool {
    true // Native assumes online; offline detected by request failure
}

/// Platform-agnostic async sleep.
#[cfg(target_arch = "wasm32")]
pub async fn sleep_ms(ms: u32) {
    gloo_timers::future::TimeoutFuture::new(ms).await;
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn sleep_ms(ms: u32) {
    tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
}
