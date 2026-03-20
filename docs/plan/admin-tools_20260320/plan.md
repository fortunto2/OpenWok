# Implementation Plan: Admin Tools (Block/Unblock + Dispute Resolution)

**Track ID:** admin-tools_20260320
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-20
**Status:** [~] In Progress

## Overview

Add user blocking and dispute resolution to the operator toolset. 3 phases: domain + migration, API endpoints with auth guards, frontend tabs. NodeOperator role is the admin role for MVP.

## Phase 1: Domain Types + Migration + Repository <!-- checkpoint:e4fc647 -->

Add dispute domain types, extend users table, implement repository methods.

### Tasks

- [x] Task 1.1: Create `migrations/0010_admin_disputes.sql` <!-- sha:e4fc647 --> — add `blocked INTEGER NOT NULL DEFAULT 0` to `users`, create `disputes` table (id, order_id, user_id, reason, status, resolution, created_at, resolved_at)
- [x] Task 1.2: Add dispute types <!-- sha:e4fc647 --> to `crates/core/src/types.rs` — `DisputeId` (id_newtype), `DisputeStatus` enum (Open, Resolved, Dismissed), `Dispute` struct, `CreateDisputeRequest`, `ResolveDisputeRequest`
- [x] Task 1.3: Add `blocked: bool` field <!-- sha:e4fc647 --> to `User` struct in `crates/core/src/types.rs`
- [x] Task 1.4: Extend `Repository` trait <!-- sha:e4fc647 --> in `crates/core/src/repo.rs` — add 5 methods: `list_users`, `set_user_blocked`, `create_dispute`, `list_disputes`, `resolve_dispute`
- [x] Task 1.5: Implement new repo methods in `crates/api/src/sqlite_repo.rs` (SqliteRepo) <!-- sha:e4fc647 -->
- [x] Task 1.6: Implement new repo methods in `crates/worker/src/d1_repo.rs` (D1Repo) <!-- sha:e4fc647 -->
- [x] Task 1.7: Update MockRepo <!-- sha:e4fc647 --> in `crates/core/src/dispatch.rs` tests to satisfy new trait methods (stub with `unimplemented!()`)

### Verification

- [x] `cargo build` succeeds for all crates
- [x] Existing tests pass (`make test`)
- [x] New migration applies cleanly on fresh DB

## Phase 2: API Endpoints + Auth Guards <!-- checkpoint:11b6953 -->

Admin-only endpoints gated by NodeOperator role. Blocked user enforcement.

### Tasks

- [x] Task 2.1: Create `crates/handlers/src/admin.rs` — admin handler module with `require_admin` helper <!-- sha:11b6953 -->
- [x] Task 2.2: Add admin endpoints in `crates/handlers/src/admin.rs` <!-- sha:11b6953 -->
- [x] Task 2.3: Add `POST /orders/{id}/dispute` to `crates/handlers/src/orders.rs` <!-- sha:11b6953 -->
- [x] Task 2.4: Add blocked-user check to auth flow — `get_active_user` helper in admin.rs <!-- sha:11b6953 -->
- [x] Task 2.5: Register admin routes in `crates/handlers/src/lib.rs` <!-- sha:11b6953 -->
- [x] Task 2.6: Add admin + dispute routes to `crates/worker/src/lib.rs` <!-- sha:11b6953 -->
- [x] Task 2.7: Write tests — block/unblock, dispute lifecycle, field persistence <!-- sha:11b6953 -->

### Verification

- [x] All admin endpoints return 403 for non-NodeOperator users
- [x] Blocked users get 403 on authenticated endpoints
- [x] Dispute lifecycle works: create → list → resolve
- [x] `make check` passes

## Phase 3: Frontend + Docs <!-- checkpoint:d167494 -->

Add Users and Disputes tabs to operator console.

### Tasks

- [x] Task 3.1: Add API client functions in `crates/frontend/src/api.rs` <!-- sha:d167494 -->
- [x] Task 3.2: Add "Users" tab to `crates/frontend/src/pages/operator.rs` <!-- sha:d167494 -->
- [x] Task 3.3: Add "Disputes" tab to `crates/frontend/src/pages/operator.rs` <!-- sha:d167494 -->
- [x] Task 3.4: Update CLAUDE.md — add new API endpoints to endpoint table, update migration table <!-- sha:d167494 -->
- [x] Task 3.5: Verify `make check` passes <!-- sha:d167494 -->

### Verification

- [x] Operator console shows Users and Disputes tabs
- [x] Block/unblock toggle works from UI
- [x] Dispute resolution works from UI
- [x] `make check` passes (clippy + fmt + tests)

## Final Verification

- [x] All acceptance criteria from spec met
- [x] Tests pass (`make test`)
- [x] Linter clean (`make clippy`)
- [x] Build succeeds (`cargo build`)
- [x] Documentation up to date (CLAUDE.md, API endpoint table)

## Context Handoff

_Summary for /build to load at session start — keeps context compact._

### Session Intent

Add admin tools (user block/unblock + dispute resolution) to close Phase 7 pilot readiness.

### Key Files

- `migrations/0010_admin_disputes.sql` — new migration
- `crates/core/src/types.rs` — DisputeId, DisputeStatus, Dispute, User.blocked
- `crates/core/src/repo.rs` — 5 new Repository trait methods
- `crates/api/src/sqlite_repo.rs` — SqliteRepo implementation
- `crates/worker/src/d1_repo.rs` — D1Repo implementation
- `crates/handlers/src/admin.rs` — new admin handler module
- `crates/handlers/src/orders.rs` — add dispute creation endpoint
- `crates/handlers/src/lib.rs` — register admin routes
- `crates/worker/src/lib.rs` — register admin routes in worker
- `crates/frontend/src/api.rs` — admin API client functions
- `crates/frontend/src/pages/operator.rs` — Users + Disputes tabs

### Decisions Made

- **NodeOperator = admin** for MVP (no separate Admin role) — consistent with federation model where node operators manage their local market
- **blocked field on users** (not a separate blocklist table) — simpler, sufficient for MVP
- **Disputes linked to orders** — every dispute has an order_id context, not free-form complaints
- **No refund integration** — dispute resolution is operational (manual Stripe refund if needed), automated refund is post-pilot

### Risks

- D1Repo parity: worker crate can't impl Repository trait (D1Database is !Send), so methods must be manually kept in sync
- Operator console has no auth guard on frontend (anyone can navigate to `/operator`) — acceptable for pilot, address post-pilot
- MockRepo in dispatch tests needs stubs for new trait methods — keep as `unimplemented!()`

## Phase 4: Review Fixes <!-- review:2026-03-20 -->

Blocked-user enforcement gap found during review. The `get_active_user` helper exists but is only called in admin endpoints and `create_dispute`. All other authenticated endpoints (restaurant CRUD, courier registration, order creation/transition) skip the blocked check.

### Tasks

- [ ] Task 4.1: Add `get_active_user` blocked check to all authenticated restaurant handlers in `crates/handlers/src/restaurants.rs` (create, update, toggle_active, add_menu_item, update_menu_item, delete_menu_item, my_restaurants)
- [ ] Task 4.2: Add `get_active_user` blocked check to all authenticated courier handlers in `crates/handlers/src/couriers.rs` (create, toggle_available, me, my_deliveries, assign_to_order)
- [ ] Task 4.3: Add `get_active_user` blocked check to authenticated order handlers in `crates/handlers/src/orders.rs` (create, transition)
- [ ] Task 4.4: Add blocked-user checks to equivalent worker routes in `crates/worker/src/lib.rs`
- [ ] Task 4.5: Add integration test verifying blocked user gets 403 on a non-admin endpoint (e.g. POST /orders)

### Verification

- [ ] Blocked users receive 403 on ALL authenticated endpoints (not just admin + dispute)
- [ ] `make check` passes
- [ ] spec.md criterion updated from [~] to [x]

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
