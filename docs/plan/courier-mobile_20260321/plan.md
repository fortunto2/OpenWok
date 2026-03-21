# Implementation Plan: Cross-Platform Frontend + Config Externalization

**Track ID:** courier-mobile_20260321
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-21
**Status:** [ ] Not Started

## Overview

Рефакторинг `crates/frontend/` в cross-platform Dioxus-приложение. Заменяем web-only deps на кроссплатформенные, выносим захардкоженные конфиги в API. 6 фаз, 14 задач.

## Phase 1: Replace gloo-net with reqwest
Механическая замена HTTP-клиента. reqwest работает на WASM (browser fetch) и native (TLS).

### Tasks
- [x] Task 1.1: Update `crates/frontend/Cargo.toml` <!-- sha:205270c --> — gloo-net → reqwest, cfg-guarded web-only deps, added dirs/open/tokio for native
- [x] Task 1.2: Rewrite `crates/frontend/src/api.rs` <!-- sha:205270c --> — reqwest::Client-based API, added api_post_raw, api_delete, owner/courier helpers
- [x] Task 1.3: Replace direct `gloo_net::Request` calls <!-- sha:205270c --> — courier.rs and owner.rs now use crate::api::* helpers

### Verification
- [x] `cargo build -p openwok-frontend --target wasm32-unknown-unknown` succeeds
- [ ] `dx serve --web` (from `crates/frontend/`) works as before
- [x] All existing tests pass (107)

## Phase 2: Abstract platform-specific code <!-- checkpoint:205270c -->
Replace `web_sys::window()` calls with cfg-guarded or platform-agnostic alternatives.

### Tasks
- [x] Task 2.1: Abstract storage in `state.rs` <!-- sha:205270c --> — cfg(wasm32) web_sys localStorage, cfg(!wasm32) file-based dirs crate
- [x] Task 2.2: Replace `web_sys::window()` calls <!-- sha:205270c --> — platform.rs: open_url(), reload_page(), sleep_ms(). Used in checkout.rs, owner.rs, courier.rs, app.rs
- [x] Task 2.3: cfg-guard analytics <!-- sha:205270c --> — PostHog no-op on native, POSTHOG_SNIPPET empty on native
- [x] Task 2.4: Abstract auth <!-- sha:205270c --> — cfg-guarded OAuth URL + callback hash parsing

### Verification
- [x] `cargo build -p openwok-frontend` (native target) succeeds
- [ ] `dx serve --web` still works without regressions
- [x] All tests pass (107)

## Phase 3: Externalize hardcoded configs
Цены из API, конфиги из environment.

### Tasks
- [x] Task 3.1: Add `GET /api/config` <!-- sha:326d650 --> endpoint in `crates/handlers/` — returns `{ "delivery_fee": "5.00", "local_ops_fee": "2.50", "federal_fee": "1.00", "api_version": "1" }`. Read from Node config in DB (fallback to defaults). Add to router in both api and worker crates
- [x] Task 3.2: Frontend fetches config <!-- sha:c5125c1 --> — `api_get("/config")` on app init, store in context signal. `pages/checkout.rs` reads fees from config context instead of hardcoded `Money::from("5.00")`. `API_BASE`: on wasm `"/api"`, on native configurable (env or compile-time const)

### Verification
- [ ] `GET /api/config` returns JSON with fees
- [ ] Checkout page shows fees from API, not hardcoded values
- [ ] Changing fees in DB/config changes what frontend displays

## Phase 4: Mobile UI additions
Bottom tab bar, mode switcher, mobile-responsive CSS.

### Tasks
- [~] Task 4.1: Add mobile layout in `app.rs` — bottom tab bar (Restaurants / Cart / Deliveries / Profile), mode switcher (Customer ↔ Courier). `AppMode` enum in state.rs. Show/hide tabs based on mode. Coexists with existing desktop header (can show both or cfg-switch)
- [ ] Task 4.2: Mobile CSS in `assets/style.css` — safe area insets (`env(safe-area-inset-*)`), 48px+ touch targets, bottom tab bar styling, full-width cards on small screens, large CTA buttons (56px). `@media (max-width: 640px)` for responsive

### Verification
- [ ] Bottom tabs visible and functional
- [ ] Mode switcher toggles between Customer/Courier views
- [ ] Touch targets 48px+ on mobile viewport

## Phase 5: Multi-platform build config
Dioxus.toml и Cargo.toml для мультиплатформенной сборки.

### Tasks
- [ ] Task 5.1: Update `crates/frontend/Dioxus.toml` — add `[bundle]` section (identifier, icons), verify `dx serve --desktop` launches desktop window with working app
- [ ] Task 5.2: Add Makefile targets — `make serve-desktop` (`cd crates/frontend && dx serve --desktop`), `make serve-mobile` (`cd crates/frontend && dx serve --ios`). Document in CLAUDE.md

### Verification
- [ ] `dx serve --desktop` opens desktop window with working app
- [ ] `dx serve --web` still works as before

## Phase 6: Cleanup & Docs

### Tasks
- [ ] Task 6.1: Remove PWA artifacts — delete `crates/frontend/public/manifest.json`, `crates/frontend/public/sw.js`, PWA icon files, remove manifest/SW references from `index.html`. Delete `docs/plan/courier-pwa_20260321/`
- [ ] Task 6.2: Update docs — CLAUDE.md: add multi-platform build commands, update workspace structure. `docs/prd.md`: replace "Courier PWA" → "Cross-platform Dioxus app". Run `make check`

### Verification
- [ ] No PWA files remain
- [ ] CLAUDE.md and PRD updated
- [ ] `make check` passes
- [ ] Web frontend works as before

## Final Verification
- [ ] All acceptance criteria from spec met
- [ ] Tests pass
- [ ] Linter clean
- [ ] Web build works (`dx serve --web`)
- [ ] Desktop build works (`dx serve --desktop`)
- [ ] Documentation up to date

## Context Handoff
_Summary for /build to load at session start — keeps context compact._

### Session Intent
Refactor existing frontend from web-only to cross-platform Dioxus app. Replace web-only deps, externalize hardcoded configs.

### Key Files
- `crates/frontend/Cargo.toml` — MODIFY: gloo-net → reqwest, conditional deps
- `crates/frontend/Dioxus.toml` — MODIFY: add bundle config
- `crates/frontend/src/api.rs` — REWRITE: gloo_net → reqwest
- `crates/frontend/src/state.rs` — MODIFY: cfg-guarded storage
- `crates/frontend/src/analytics.rs` — MODIFY: cfg-guard PostHog
- `crates/frontend/src/app.rs` — MODIFY: add bottom tabs, mode switcher, config context
- `crates/frontend/src/pages/auth.rs` — MODIFY: cfg-guard web_sys usage
- `crates/frontend/src/pages/checkout.rs` — MODIFY: fees from config, open::that for Stripe
- `crates/frontend/src/pages/courier.rs` — MODIFY: use api.rs helpers, signal refresh
- `crates/frontend/src/pages/owner.rs` — MODIFY: use api.rs helpers, signal refresh
- `crates/frontend/assets/style.css` — MODIFY: add mobile CSS
- `crates/handlers/src/lib.rs` — MODIFY: add /api/config route
- `crates/api/src/main.rs` — MODIFY: register /api/config

### Decisions Made
- **Один крейт, два таргета** — не копируем код. `dx serve --web` и `dx serve --desktop` из одного крейта
- **reqwest everywhere** — работает на WASM (browser fetch) и native (TLS). Единый HTTP-клиент
- **cfg guards по target_arch** — `wasm32` для web-specific, native для file-based storage
- **Цены из API** — `GET /api/config` вместо хардкода. Node operator может менять fees
- **Bottom tabs для mobile** — CSS responsive, показываем на маленьких экранах
- **PostHog no-op на native** — JS snippet не работает. Native SDK — отдельный трек

### Risks
- **reqwest на WASM** — работает, но может быть чуть больше bundle size чем gloo-net. Мониторить
- **uuid `js` feature** — нужен на WASM для crypto random, не нужен на native. Conditional в Cargo.toml
- **dx serve --desktop** — Dioxus desktop использует Tao/Wry WebView. Может требовать GTK на Linux
- **Существующий worker crate** — не трогаем. Worker собирается отдельно с D1Repo

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
