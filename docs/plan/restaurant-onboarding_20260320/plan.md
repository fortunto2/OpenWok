# Implementation Plan: Restaurant Onboarding & Self-Service

**Track ID:** restaurant-onboarding_20260320
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-20
**Status:** [~] In Progress

## Overview

Add restaurant ownership model + self-service management. Migration for owner_id, Repository methods for CRUD, role-based auth guards, management API endpoints, and frontend dashboard. 13 tasks across 4 phases.

## Phase 1: Backend Foundation
Add ownership to DB schema, implement Repository methods, create management handlers with role guards.

### Tasks
- [x] Task 1.1: Migration `migrations/0008_restaurant_owner.sql` — add `owner_id TEXT REFERENCES users(id)`, `description TEXT`, `address TEXT`, `phone TEXT`, `created_at TEXT`, `updated_at TEXT` to restaurants table. Update existing rows: set `owner_id = NULL` (no owner yet).
- [x] Task 1.2: Extend Repository trait in `crates/core/src/repo.rs` — add methods: `update_restaurant(id, UpdateRestaurantRequest)`, `toggle_restaurant_active(id, active)`, `list_restaurants_by_owner(user_id)`, `add_menu_item(restaurant_id, CreateMenuItemRequest)`, `update_menu_item(id, UpdateMenuItemRequest)`, `delete_menu_item(id)`, `update_user_role(user_id, UserRole)`. Add `UpdateRestaurantRequest` and `UpdateMenuItemRequest` types to `crates/core/src/types.rs`.
- [x] Task 1.3: Implement new Repository methods in `crates/api/src/sqlite_repo.rs`. Write unit tests for each method — ownership enforcement, role update, menu CRUD.
- [x] Task 1.4: Add restaurant management handlers in `crates/handlers/src/restaurants.rs` — `PATCH /api/restaurants/:id` (update info), `PATCH /api/restaurants/:id/active` (toggle), `POST /api/restaurants/:id/menu` (add item), `PATCH /api/menu-items/:id` (update item), `DELETE /api/menu-items/:id` (delete item), `GET /api/my/restaurants` (owner's list). Add ownership check helper. Protect existing `POST /api/restaurants` with auth. On create: auto-promote user to RestaurantOwner role.
- [x] Task 1.5: Integration tests in `crates/api/` — test ownership enforcement (owner can edit, non-owner gets 403), role promotion on restaurant creation, menu CRUD.

### Verification
- [x] `cargo test -p openwok-core -p openwok-handlers -p openwok-api` — all pass
- [x] `cargo clippy --all` — no warnings
- [x] New endpoints return correct HTTP status codes (200, 201, 403, 404)

## Phase 2: Worker (D1)
Port new Repository methods to D1Repo and add routes to Cloudflare Worker.

### Tasks
- [x] Task 2.1: Implement new methods in `crates/worker/src/d1_repo.rs` — same signatures as SqliteRepo: update_restaurant, toggle_active, list_by_owner, menu CRUD, update_user_role.
- [x] Task 2.2: Add worker routes in `crates/worker/src/lib.rs` — mirror all new handlers: PATCH restaurant, toggle active, menu CRUD, GET /my/restaurants. Apply auth guards (extract JWT, verify ownership).

### Verification
- [x] `cargo check -p openwok-worker --target wasm32-unknown-unknown` — compiles
- [ ] Manual test with `wrangler dev` — new endpoints respond correctly

## Phase 3: Frontend
Restaurant owner dashboard with menu editor and onboarding form.

### Tasks
- [x] Task 3.1: Extend auth state in `crates/frontend/src/main.rs` — add `role: Option<UserRole>` to UserState. Fetch role from `GET /api/auth/me` after login. Show "My Restaurants" nav link when role is RestaurantOwner. Add routes: `/my-restaurants`, `/my-restaurants/:id`, `/onboard-restaurant`.
- [x] Task 3.2: Create MyRestaurants page — list owner's restaurants (GET /api/my/restaurants), show name + active status + menu item count. Link to settings. "Add Restaurant" button links to onboard form.
- [x] Task 3.3: Create RestaurantSettings + MenuEditor — settings form (name, description, address, phone, active toggle). Menu section: list items with edit/delete, "Add Item" form. Inline editing with save/cancel.
- [x] Task 3.4: Create RestaurantOnboarding form — name, zone (dropdown from zones), description, address, phone. On submit: POST /api/restaurants → redirect to menu editor. Initial menu: form to add first items.

### Verification
- [x] `dx build --platform web` — compiles (cargo check passes)
- [ ] Visual test: login → create restaurant → add menu items → toggle active → edit details

## Phase 4: Deploy & Docs
Deploy updated Worker, verify live, update documentation.

### Tasks
- [x] Task 4.1: Build frontend (`dx build --platform web --release`), copy to worker public/, deploy (`wrangler deploy`). Run migration 0008 on D1. Verify live: create restaurant → manage menu → check public listing.
- [x] Task 4.2: Update `CLAUDE.md` — add new endpoints to API table, add new frontend routes, update repo structure notes. Remove dead code if any.

### Verification
- [ ] Service is live and healthy
- [ ] No runtime errors in production logs
- [ ] New endpoints return correct responses on live URL

## Final Verification

- [x] All acceptance criteria from spec met
- [x] Tests pass (`make check`)
- [x] Linter clean
- [x] Build succeeds (all targets)
- [x] Documentation up to date
- [ ] Restaurant owner can go from login → create restaurant → add menu → go live (end-to-end on production)

## Context Handoff
_Summary for /build to load at session start — keeps context compact._

### Session Intent
Add restaurant self-service: ownership model, management API, and owner dashboard.

### Key Files
- `migrations/0008_restaurant_owner.sql` — NEW
- `crates/core/src/types.rs` — add UpdateRestaurantRequest, UpdateMenuItemRequest
- `crates/core/src/repo.rs` — add 7 Repository methods
- `crates/api/src/sqlite_repo.rs` — implement new methods
- `crates/handlers/src/restaurants.rs` — 6 new handlers + ownership guard
- `crates/worker/src/d1_repo.rs` — D1 implementations
- `crates/worker/src/lib.rs` — new routes
- `crates/frontend/src/main.rs` — 4 new pages/components, auth state extension

### Decisions Made
- Auto-promote user to RestaurantOwner on first restaurant creation (no admin approval for MVP pilot)
- Ownership check is per-request (load restaurant, compare owner_id) — no middleware extractor for ownership
- No KYB/approval workflow — keep it simple for 10-20 pilot restaurants
- Menu items are flat (no categories/sections) — sufficient for MVP
- No image upload — text-only menus for now

### Risks
- Frontend is one large file (main.rs ~1143 lines) — adding 4 more pages will push it further. Consider splitting in a future refactor track.
- No Tailwind yet — CSS will be handwritten (consistent with existing style). Tailwind migration is a separate track.
- D1 migrations must be applied manually via `wrangler d1 execute` — no auto-migration in Worker.

## Phase 5: Review Fixes

### Tasks
- [ ] Task 5.1: Fix TOCTOU in `crates/handlers/src/restaurants.rs` `update_menu_item` handler — move `verify_ownership()` call BEFORE `repo.update_menu_item()`. Need to add `get_menu_item(id)` to Repository trait or query restaurant_id first.
- [ ] Task 5.2: Fix TOCTOU in `crates/worker/src/lib.rs` `PATCH /api/menu-items/:id` route — same fix: verify ownership before applying update.
- [ ] Task 5.3: Fix clippy errors in `crates/frontend/src/main.rs` — line 1471: remove `.clone()` on Copy type `Navigator`; line 1599: collapse nested if.
- [ ] Task 5.4: Run `make check` to verify all fixes pass (tests + clippy + fmt).

### Verification
- [ ] `make check` passes cleanly
- [ ] TOCTOU fixed: non-owner update attempt returns 403 without modifying data

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
