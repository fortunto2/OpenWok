# Implementation Plan: OpenWok Mobile App (Dioxus Native)

**Track ID:** courier-mobile_20260321
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-21
**Status:** [ ] Not Started

## Overview

Создаём `crates/mobile/` — единое Dioxus-приложение для iOS с двумя режимами (customer / courier). Портируем UI-логику из web-frontend, заменяя web-only deps на cross-platform аналоги. 5 фаз, 15 задач.

## Phase 1: Scaffold & Hello World
Создать крейт, настроить Dioxus для mobile, запустить на iOS симуляторе.

### Tasks
- [x] Task 1.1: Create `crates/mobile/Cargo.toml` <!-- sha:73ffd7a --> — deps: `dioxus = { version = "0.7", features = ["router"] }`, `reqwest = { version = "0.12", features = ["json", "rustls-tls"] }`, `openwok-core = { path = "../core" }`, `serde`, `serde_json`, `dirs` (JWT storage), `open` (system browser for OAuth). No `gloo-net`, `web-sys`, `js-sys`, `wasm-bindgen`
- [x] Task 1.2: Create `crates/mobile/Dioxus.toml` <!-- sha:73ffd7a --> — `[application]` name = "openwok-mobile", `[bundle]` identifier = "co.superduperai.openwok", icons
- [x] Task 1.3: Add `"crates/mobile"` to workspace `members` in root `Cargo.toml`. Create `crates/mobile/src/main.rs` <!-- sha:73ffd7a --> — minimal Dioxus app with "OpenWok" text
- [x] Task 1.4: Install iOS toolchains (`rustup target add aarch64-apple-ios aarch64-apple-ios-sim`) <!-- sha:73ffd7a --> — targets installed. `dx serve --ios` blocked by missing Xcode.app (only CommandLineTools installed). Dev via desktop mode.

### Verification
- [x] `cargo build -p openwok-mobile` succeeds
- [ ] `dx serve --ios` shows hello world in iOS simulator — BLOCKED: needs Xcode.app install

## Phase 2: Shared Infrastructure
API client, auth, storage — cross-platform foundation.

### Tasks
- [x] Task 2.1: Create shared modules <!-- sha:510563e --> — `config.rs` (API_BASE = `"https://openwok.superduperai.co/api"`, Supabase URL/key), `storage.rs` (JWT file persistence via `directories::ProjectDirs`), `state.rs` (UserState signal + AppMode enum {Customer, Courier})
- [x] Task 2.2: Create `api.rs` <!-- sha:7724b4b --> — `reqwest::Client`-based: `api_get<T>`, `api_post_json<T>`, `api_patch_json`. Same function signatures as web `api.rs`, but reqwest. Include data fetchers: `fetch_restaurants`, `fetch_restaurant`, `place_order`, `fetch_order`, `fetch_courier_me`, `fetch_my_deliveries`
- [x] Task 2.3: Create `auth.rs` <!-- sha:8540c1d --> — Supabase Google OAuth: build auth URL → `open::that(url)` (system browser) → handle deep link callback `openwok://auth/callback?access_token=...` → store JWT via storage.rs. Login/Logout via Dioxus signals

### Verification
- [ ] API client fetches `GET /api/restaurants` from production and returns data
- [ ] JWT save/load round-trips correctly
- [ ] OAuth URL opens in system browser

## Phase 3: Customer Mode UI
Рестораны → меню → корзина → чекаут → трекинг. Портируем из web frontend, адаптируя под mobile.

### Tasks
- [x] Task 3.1: Create `app.rs` <!-- sha:1773d10 --> — Route enum (9 routes), App component with auth guard, Layout with mobile bottom tab bar (Restaurants / Orders / Profile), mode switcher (Customer ↔ Courier) in profile/settings. No desktop nav-links header
- [~] Task 3.2: Create customer pages — `pages/restaurants.rs` (RestaurantList + RestaurantMenu + CartPanel, adapted from web), `pages/checkout.rs` (6-line pricing breakdown, Stripe redirect via system browser), `pages/order.rs` (OrderTracking with status timeline). Port RSX from `crates/frontend/src/pages/`, replace `gloo_net::Request` with `crate::api::*` calls, remove `web_sys` references
- [ ] Task 3.3: Create `assets/style.css` — mobile-first CSS: safe area insets (`env(safe-area-inset-*)`), 48px+ touch targets, bottom tab bar, full-width cards, sticky headers, large buttons (56px CTA), no desktop-only styles

### Verification
- [ ] Restaurant list loads and displays on iOS simulator
- [ ] Can browse menu, add to cart, see pricing breakdown
- [ ] Checkout redirects to Stripe (system browser)
- [ ] Order tracking shows status timeline

## Phase 4: Courier Mode UI
Регистрация + дашборд с toggle/mark delivered + auto-refresh.

### Tasks
- [ ] Task 4.1: Create courier pages — `pages/courier.rs`: RegisterCourier (name + zone dropdown, 44px+ inputs, full-width submit), MyDeliveries (availability toggle as large switch, active delivery card with 56px "Mark Delivered" button, delivery history cards). Port from `crates/frontend/src/pages/courier.rs`, replace API calls
- [ ] Task 4.2: Add auto-refresh — poll `GET /api/my/deliveries` every 15s when courier is available (`use_future` with `tokio::time::sleep`). Add pull-to-refresh pattern. Loading/error states with retry buttons
- [ ] Task 4.3: Build release — `dx bundle --platform ios`, verify .app on simulator, document TestFlight upload steps

### Verification
- [ ] Courier registration works from mobile
- [ ] Availability toggle calls API and updates UI
- [ ] "Mark Delivered" transitions order status
- [ ] New deliveries appear within 15s
- [ ] `dx bundle --platform ios` produces valid .app

## Phase 5: Cleanup & Docs
Удалить PWA-артефакты, обновить документацию.

### Tasks
- [ ] Task 5.1: Remove PWA artifacts — delete `crates/frontend/public/manifest.json`, delete `docs/plan/courier-pwa_20260321/` directory
- [ ] Task 5.2: Update docs — `docs/prd.md`: replace "Courier PWA" → "OpenWok Mobile (Dioxus native)" in Phase 7. CLAUDE.md: add `crates/mobile/` to workspace structure, add mobile build commands. `planning/ROADMAP.md`: replace PWA references
- [ ] Task 5.3: Run `make check` — all tests pass, clippy clean, fmt clean. Verify web frontend still builds (`dx build --platform web` from `crates/frontend/`)

### Verification
- [ ] No PWA files remain
- [ ] CLAUDE.md and PRD reflect mobile app
- [ ] `make check` passes
- [ ] Web frontend unaffected

## Final Verification
- [ ] All acceptance criteria from spec met
- [ ] Tests pass
- [ ] Linter clean
- [ ] Both builds succeed: web frontend + mobile app
- [ ] Documentation up to date
- [ ] Mobile app runs on iOS simulator with both modes

## Context Handoff
_Summary for /build to load at session start — keeps context compact._

### Session Intent
Build a unified Dioxus native mobile app (iOS) with customer and courier modes, replacing the PWA approach.

### Key Files
- `crates/mobile/Cargo.toml` — NEW: dioxus 0.7, reqwest, openwok-core, directories, open
- `crates/mobile/Dioxus.toml` — NEW: bundle config, iOS URL scheme
- `crates/mobile/src/main.rs` — NEW: entry point
- `crates/mobile/src/app.rs` — NEW: Route enum (9 routes), App, Layout with bottom tabs, mode switcher
- `crates/mobile/src/api.rs` — NEW: reqwest-based API client (port from web api.rs)
- `crates/mobile/src/auth.rs` — NEW: Supabase OAuth + deep link callback
- `crates/mobile/src/storage.rs` — NEW: file-based JWT persistence
- `crates/mobile/src/config.rs` — NEW: API_BASE, Supabase config
- `crates/mobile/src/state.rs` — NEW: UserState, CartState, AppMode
- `crates/mobile/src/pages/restaurants.rs` — NEW: RestaurantList + RestaurantMenu (port from web)
- `crates/mobile/src/pages/checkout.rs` — NEW: Checkout with pricing (port from web)
- `crates/mobile/src/pages/order.rs` — NEW: OrderTracking (port from web)
- `crates/mobile/src/pages/courier.rs` — NEW: RegisterCourier + MyDeliveries (port from web)
- `crates/mobile/assets/style.css` — NEW: mobile-first CSS
- `Cargo.toml` — add mobile to workspace members
- `crates/frontend/public/manifest.json` — DELETE
- `docs/plan/courier-pwa_20260321/` — DELETE

### Decisions Made
- **Одно приложение, два режима** — Customer ↔ Courier переключатель. На пилоте курьеры = клиенты. Один апп, один логин.
- **Отдельный крейт `crates/mobile/`** — web frontend (`crates/frontend/`) остаётся. У них разные deps: web = gloo-net + web-sys, mobile = reqwest + directories. Общий код — `openwok-core` types.
- **Портирование, не шаринг UI** — RSX копируем и адаптируем (заменяем API calls, убираем web_sys). Не пытаемся делать shared UI crate — это over-engineering на данном этапе.
- **9 из 15 роутов** — пропускаем desktop-only: operator, economics, my-restaurants (owner), onboard-restaurant, restaurant-settings, admin. Operator/admin/owner = веб.
- **Bottom tab bar** — мобильная навигация вместо desktop header. Tabs: Restaurants, Orders, Profile/Settings.
- **System browser OAuth** — Google рекомендует, безопаснее embedded WebView. Deep link для callback.
- **iOS first** — macOS dev, toolchain ready. Android — follow-up track.

### Risks
- **Deep link auth** — iOS URL scheme нужно тестировать с Supabase redirect URL. Fallback: QR-код логин или manual token.
- **Dioxus 0.7 mobile maturity** — WebView rendering может иметь edge cases. Тестировать каждый экран.
- **CORS** — mobile origin ≠ web origin. Может потребоваться `Access-Control-Allow-Origin: *` на Worker.
- **Stripe checkout on mobile** — redirect в system browser для оплаты, return URL через deep link. Нужно тестировать полный flow.
- **openwok-core на ARM64** — чистый Rust, должно компилироваться, но verify.
- **Размер бинаря** — reqwest + rustls может добавить мегабайты. Мониторить.

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
