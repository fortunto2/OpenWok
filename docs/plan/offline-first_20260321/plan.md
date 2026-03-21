# Implementation Plan: Offline-First SQLite Cache Layer

**Track ID:** offline-first_20260321
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-21
**Status:** [ ] Not Started

## Overview

Добавляем локальную SQLite в браузер/мобилку как кеш-слой. Курьер работает offline, изменения синхронизируются при reconnect. 4 фазы, 12 задач.

## Phase 1: Local SQLite Setup
Подключить sqlite-wasm-rs (WASM) и rusqlite (native), создать локальную схему, абстрагировать доступ.

### Tasks
- [~] Task 1.1: Add deps to `crates/frontend/Cargo.toml` — WASM: `sqlite-wasm-rs = "0.5"`, `sqlite-wasm-vfs = "0.2"`. Native: `rusqlite = "0.34"` (уже в workspace). Feature-gate через `cfg(target_arch)`
- [ ] Task 1.2: Create `crates/frontend/src/local_db.rs` — platform-abstracted local database module. `cfg(wasm32)`: open SQLite via `sqlite-wasm-rs` + IndexedDB VFS. `cfg(!wasm32)`: open via `rusqlite` from `dirs::data_dir()`. Public API: `init_local_db() -> LocalDb`, `LocalDb::execute()`, `LocalDb::query()`
- [ ] Task 1.3: Create local schema migration in `local_db.rs` — run on `init_local_db()`. Tables: `couriers` (id, name, available), `orders` (id, restaurant_id, courier_id, customer_address, status, food_total, created_at, updated_at), `order_items` (id, order_id, name, quantity, unit_price), `pending_actions` (id INTEGER PRIMARY KEY, action TEXT, payload TEXT, created_at TEXT). Subset of server schema, D1-compatible SQL

### Verification
- [ ] `cargo build -p openwok-frontend --target wasm32-unknown-unknown` succeeds with sqlite-wasm-rs
- [ ] `cargo build -p openwok-frontend` (native) succeeds with rusqlite
- [ ] Unit test: init_local_db + create table + insert + query round-trip

## Phase 2: Sync Engine
Pull (сервер → локальная база) и Push (outbox → API) синхронизация.

### Tasks
- [ ] Task 2.1: Create `crates/frontend/src/sync.rs` — sync engine module. `pull_deliveries(db, api)`: fetch `GET /api/my/deliveries` + `GET /api/couriers/me`, upsert into local SQLite. `push_pending(db, api)`: read `pending_actions` table, execute each via API (PATCH /orders/{id}/status), delete on success
- [ ] Task 2.2: Add online detection in `platform.rs` — `is_online() -> bool`: WASM uses `web_sys::window().navigator().on_line()`, native always returns `true`. Add `on_connectivity_change(callback)` for WASM (`online`/`offline` events)
- [ ] Task 2.3: Create outbox helpers in `sync.rs` — `queue_action(db, action, payload)`: insert into `pending_actions`. `pending_count(db) -> usize`. Actions: `"mark_delivered"` with payload `{"order_id": "..."}`. On reconnect → `push_pending` drains queue

### Verification
- [ ] Pull sync: API data appears in local SQLite
- [ ] Push sync: queued actions execute via API on reconnect
- [ ] Pending count reflects queued offline actions

## Phase 3: Wire into Courier UI
Подключить offline-aware data loading и offline actions к MyDeliveries.

### Tasks
- [ ] Task 3.1: Initialize LocalDb in `app.rs` — call `init_local_db()` on startup, store as Dioxus context (`Signal<Option<LocalDb>>`). Start background sync loop: every 15s if online → `pull_deliveries` + `push_pending`
- [ ] Task 3.2: Update `pages/courier.rs` MyDeliveries — load from local SQLite first (instant), then refresh from API when online. Show data immediately from cache. "Mark Delivered" button: if online → API call + update local DB. If offline → `queue_action` + update local DB optimistically
- [ ] Task 3.3: Add connectivity indicator — show "Offline" badge in Layout when `!is_online()`. Show pending actions count badge on Deliveries tab. Toast "Back online — syncing..." when reconnect detected

### Verification
- [ ] MyDeliveries loads instantly from cache (no loading spinner on repeat visits)
- [ ] "Mark Delivered" works offline (saved to outbox, UI updates)
- [ ] On reconnect, outbox drains and server reflects changes
- [ ] Offline badge visible when disconnected

## Phase 4: Docs & Cleanup

### Tasks
- [ ] Task 4.1: Update CLAUDE.md — add local_db.rs, sync.rs to workspace structure. Document offline architecture. Add `sqlite-wasm-rs` to tech stack
- [ ] Task 4.2: Run `make check` — all tests pass, clippy clean, WASM build works. Remove dead code

### Verification
- [ ] CLAUDE.md reflects offline-first architecture
- [ ] `make check` passes
- [ ] WASM build succeeds

## Final Verification
- [ ] All acceptance criteria from spec met
- [ ] Tests pass
- [ ] Linter clean
- [ ] Courier dashboard works offline
- [ ] Outbox syncs on reconnect
- [ ] Documentation up to date

## Context Handoff
_Summary for /build to load at session start — keeps context compact._

### Session Intent
Add offline-first SQLite cache layer for courier dashboard — local DB in browser via sqlite-wasm-rs, outbox pattern for offline actions, sync on reconnect.

### Key Files
- `crates/frontend/Cargo.toml` — MODIFY: add sqlite-wasm-rs (wasm32), rusqlite (native)
- `crates/frontend/src/local_db.rs` — NEW: platform-abstracted local SQLite (init, execute, query)
- `crates/frontend/src/sync.rs` — NEW: pull sync (API→local), push sync (outbox→API), online detection
- `crates/frontend/src/platform.rs` — MODIFY: add is_online(), on_connectivity_change()
- `crates/frontend/src/app.rs` — MODIFY: init LocalDb context, start sync loop
- `crates/frontend/src/pages/courier.rs` — MODIFY: load from cache first, offline-aware actions
- `crates/frontend/src/app.rs` — MODIFY: add offline badge to Layout

### Decisions Made
- **sqlite-wasm-rs over sql.js** — Rust-native, same API as rusqlite, no JS bridge needed
- **IndexedDB VFS over OPFS** — wider browser support (OPFS requires secure context + headers)
- **Outbox pattern over CRDT** — simple, sufficient for single-courier single-device. No concurrent edits
- **Last-write-wins** — server is source of truth, local is cache. Server wins on conflict
- **Courier scope only** — customer flow needs Stripe (online). Restaurant owner has reliable connection. Couriers are the ones who lose signal
- **No Service Worker** — WASM binary cached by browser. Offline logic lives in Rust, not in SW

### Risks
- **sqlite-wasm-rs bundle size** — SQLite WASM adds ~300-500KB. Monitor total bundle
- **IndexedDB limits** — 50MB+ available, but varies per browser. For courier data (tens of orders) this is fine
- **rusqlite in frontend crate** — adds native SQLite dependency to frontend. Only compiled on native targets via cfg
- **Async SQLite on WASM** — sqlite-wasm-rs may need `wasm-bindgen-futures` for async ops. May need sync wrappers
- **First load** — on very first visit, local DB is empty. Show "Loading..." until first pull sync completes

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
