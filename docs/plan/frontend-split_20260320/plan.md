# Implementation Plan: Frontend Module Split + Pre-commit Hooks

**Track ID:** frontend-split_20260320
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-20
**Status:** [~] In Progress

## Overview

Extract the 2060-line `main.rs` monolith into 11 focused modules. Add pre-commit hooks for fmt + clippy. Pure refactor — zero behavior changes.

## Target Structure

```
crates/frontend/src/
├── main.rs              (~10 lines: mod declarations, fn main)
├── app.rs               (~100 lines: Route enum, App, Layout)
├── state.rs             (~50 lines: UserState, CartState, CartItem, JWT storage helpers)
├── analytics.rs         (~45 lines: posthog_capture, posthog_capture_with_props, POSTHOG_SNIPPET)
├── api.rs               (~130 lines: API_BASE, auth_header, api_get/post/patch, all fetch_* fns, cart_total)
├── pages/
│   ├── mod.rs           (~10 lines: pub mod declarations)
│   ├── home.rs          (~15 lines: Home component)
│   ├── auth.rs          (~110 lines: Login, AuthCallback)
│   ├── restaurants.rs   (~150 lines: RestaurantList, RestaurantCard, RestaurantMenu, CartPanel)
│   ├── checkout.rs      (~180 lines: Checkout)
│   ├── order.rs         (~190 lines: OrderTracking, OrderSuccess)
│   ├── economics.rs     (~75 lines: PublicEconomicsPage)
│   ├── operator.rs      (~320 lines: OperatorConsole, MetricsPanel, OrderRow)
│   ├── owner.rs         (~440 lines: MyRestaurants, OnboardRestaurant, RestaurantSettings)
│   └── courier.rs       (~230 lines: RegisterCourier, MyDeliveries)
```

## Phase 1: Extract Shared Modules

Extract foundational modules that page components depend on.

### Tasks
- [x] Task 1.1: Create `crates/frontend/src/state.rs` — move `UserState`, `CartState`, `CartItem`, `get_jwt_from_storage`, `get_local_storage`, `save_jwt_to_storage`, `clear_jwt_from_storage` (lines 11-55). Make all items `pub`. <!-- sha:599825b -->
- [x] Task 1.2: Create `crates/frontend/src/analytics.rs` — move `posthog_capture`, `posthog_capture_with_props`, `POSTHOG_SNIPPET` (lines 57-99). Make all items `pub`. <!-- sha:599825b -->
- [x] Task 1.3: Create `crates/frontend/src/api.rs` — move `API_BASE`, `auth_header`, `api_get`, `api_post_json`, `api_patch_json`, all `fetch_*` functions, `assign_courier`, `transition_order`, `cart_total` (lines 338-476). Make all items `pub`. Import `get_jwt_from_storage` from `crate::state`. <!-- sha:599825b -->
- [x] Task 1.4: Add `mod state; mod analytics; mod api;` to `main.rs`, replace inline code with `use` imports. Verify `cargo build -p openwok-frontend` compiles. <!-- sha:599825b -->

### Verification
- [x] `cargo build -p openwok-frontend` succeeds
- [x] `cargo clippy -p openwok-frontend` clean

## Phase 2: Extract Page Components <!-- checkpoint:599825b -->

Move each page group into `pages/` directory.

### Tasks
- [x] Task 2.1: Create `crates/frontend/src/pages/mod.rs` with `pub mod` for all page modules. <!-- sha:599825b -->
- [x] Task 2.2: Create `pages/home.rs` — move `Home` component (~line 217-225). <!-- sha:599825b -->
- [x] Task 2.3: Create `pages/auth.rs` — move `Login`, `AuthCallback` (lines 229-336). Import from `crate::state`, `crate::api`. <!-- sha:599825b -->
- [x] Task 2.4: Create `pages/restaurants.rs` — move `RestaurantList`, `RestaurantCard`, `RestaurantMenu`, `CartPanel` (lines 479-628). Import from `crate::state`, `crate::api`, `crate::analytics`. <!-- sha:599825b -->
- [x] Task 2.5: Create `pages/checkout.rs` — move `Checkout` (lines 631-806). Import from `crate::state`, `crate::api`, `crate::analytics`. <!-- sha:599825b -->
- [x] Task 2.6: Create `pages/order.rs` — move `OrderTracking`, `OrderSuccess` (lines 809-993). Import from `crate::api`, `crate::analytics`. <!-- sha:599825b -->
- [x] Task 2.7: Create `pages/economics.rs` — move `PublicEconomicsPage` (lines 996-1066). Import from `crate::api`. <!-- sha:599825b -->
- [x] Task 2.8: Create `pages/operator.rs` — move `OperatorConsole`, `MetricsPanel`, `OrderRow` (lines 1069-1387). Import from `crate::api`. <!-- sha:599825b -->
- [x] Task 2.9: Create `pages/owner.rs` — move `MyRestaurants`, `OnboardRestaurant`, `RestaurantSettings` (lines 1390-1824). Import from `crate::state`, `crate::api`, `crate::analytics`. <!-- sha:599825b -->
- [x] Task 2.10: Create `pages/courier.rs` — move `RegisterCourier`, `MyDeliveries` (lines 1827-2057). Import from `crate::state`, `crate::api`. <!-- sha:599825b -->

### Verification
- [x] `cargo build -p openwok-frontend` succeeds
- [x] All page components accessible from `crate::pages::*`

## Phase 3: Finalize main.rs + App Module <!-- checkpoint:599825b -->

Slim down main.rs to mod declarations, extract App/Layout/Route.

### Tasks
- [x] Task 3.1: Create `crates/frontend/src/app.rs` — move `Route` enum, `App` component, `Layout` component (lines 101-225). Import page components from `crate::pages::*`, state from `crate::state`, analytics from `crate::analytics`. <!-- sha:599825b -->
- [x] Task 3.2: Reduce `main.rs` to: mod declarations (`mod state; mod analytics; mod api; mod app; mod pages;`) + `fn main()` (~10 lines). <!-- sha:599825b -->
- [x] Task 3.3: Run `cargo fmt -p openwok-frontend` and `cargo clippy -p openwok-frontend`. Fix any warnings. <!-- sha:599825b -->

### Verification
- [x] `main.rs` is under 30 lines
- [x] No file exceeds 500 lines
- [x] `cargo build -p openwok-frontend` succeeds with zero warnings
- [x] `cargo test --workspace` — tests pass

## Phase 4: Pre-commit Hooks

Add harness guardrails (3 retro recommendations).

### Tasks
- [ ] Task 4.1: Create `.githooks/pre-commit` script: `cargo fmt --check` + `cargo clippy --all -- -D warnings`. Make executable.
- [ ] Task 4.2: Add `Makefile` target `setup-hooks` that runs `git config core.hooksPath .githooks`. Document in README or CLAUDE.md.

### Verification
- [ ] Pre-commit hook blocks commits with fmt/clippy issues
- [ ] Hook path is committed and portable (`.githooks/`, not `.git/hooks/`)

## Phase 5: Docs & Cleanup

### Tasks
- [ ] Task 5.1: Update CLAUDE.md — add frontend module structure, `make setup-hooks` command
- [ ] Task 5.2: Remove any dead imports or unused code from refactored files
- [ ] Task 5.3: Run full verification: `cargo test --workspace && cargo clippy --all && cargo fmt --check`

### Verification
- [ ] CLAUDE.md reflects current project state
- [ ] Linter clean, tests pass

## Final Verification

- [ ] All acceptance criteria from spec met
- [ ] Tests pass (101 tests, 0 failures)
- [ ] Linter clean
- [ ] Build succeeds
- [ ] `main.rs` under 30 lines, no file over 500 lines
- [ ] Pre-commit hooks working

## Context Handoff

_Summary for /build to load at session start — keeps context compact._

### Session Intent

Split the 2060-line frontend monolith (`crates/frontend/src/main.rs`) into 11 modules and add pre-commit hooks.

### Key Files

- `crates/frontend/src/main.rs` — source monolith (will become ~10 lines)
- `crates/frontend/src/state.rs` — NEW: auth + cart state
- `crates/frontend/src/analytics.rs` — NEW: PostHog helpers
- `crates/frontend/src/api.rs` — NEW: API client + data fetchers
- `crates/frontend/src/app.rs` — NEW: Route enum + App + Layout
- `crates/frontend/src/pages/*.rs` — NEW: 8 page modules
- `.githooks/pre-commit` — NEW: fmt + clippy hook
- `Makefile` — add `setup-hooks` target
- `CLAUDE.md` — update workspace structure section

### Decisions Made

- **Pages grouped by domain** (not one-file-per-component) — keeps related components together (e.g., `operator.rs` has Console + MetricsPanel + OrderRow)
- **`.githooks/` over `.git/hooks/`** — committed to repo, portable across clones
- **No new dependencies** — pure file reorganization
- **`pub` visibility on shared items** — modules need cross-access (state, api, analytics)

### Risks

- Dioxus `#[derive(Routable)]` macro requires all component functions to be in scope where `Route` is defined — may need `use crate::pages::*` in `app.rs`
- `asset!()` macro paths are relative to crate root — should work unchanged but verify after move
- Cart/User context signals accessed via `use_context` — no prop changes needed, but verify each page compiles

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
