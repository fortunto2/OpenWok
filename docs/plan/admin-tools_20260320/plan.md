# Implementation Plan: Admin Tools (Block/Unblock + Dispute Resolution)

**Track ID:** admin-tools_20260320
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-20
**Status:** [ ] Not Started

## Overview

Add user blocking and dispute resolution to the operator toolset. 3 phases: domain + migration, API endpoints with auth guards, frontend tabs. NodeOperator role is the admin role for MVP.

## Phase 1: Domain Types + Migration + Repository

Add dispute domain types, extend users table, implement repository methods.

### Tasks

- [x] Task 1.1: Create `migrations/0010_admin_disputes.sql` — add `blocked INTEGER NOT NULL DEFAULT 0` to `users`, create `disputes` table (id, order_id, user_id, reason, status, resolution, created_at, resolved_at)
- [x] Task 1.2: Add dispute types to `crates/core/src/types.rs` — `DisputeId` (id_newtype), `DisputeStatus` enum (Open, Resolved, Dismissed), `Dispute` struct, `CreateDisputeRequest`, `ResolveDisputeRequest`
- [x] Task 1.3: Add `blocked: bool` field to `User` struct in `crates/core/src/types.rs`
- [~] Task 1.4: Extend `Repository` trait in `crates/core/src/repo.rs` — add 5 methods: `list_users`, `set_user_blocked`, `create_dispute`, `list_disputes`, `resolve_dispute`
- [ ] Task 1.5: Implement new repo methods in `crates/api/src/sqlite_repo.rs` (SqliteRepo)
- [ ] Task 1.6: Implement new repo methods in `crates/worker/src/d1_repo.rs` (D1Repo)
- [ ] Task 1.7: Update MockRepo in `crates/core/src/dispatch.rs` tests to satisfy new trait methods (stub with `unimplemented!()`)

### Verification

- [ ] `cargo build` succeeds for all crates
- [ ] Existing tests pass (`make test`)
- [ ] New migration applies cleanly on fresh DB

## Phase 2: API Endpoints + Auth Guards

Admin-only endpoints gated by NodeOperator role. Blocked user enforcement.

### Tasks

- [ ] Task 2.1: Create `crates/handlers/src/admin.rs` — admin handler module with `require_admin` helper (loads user from repo by supabase_id, checks role == NodeOperator && !blocked, returns 403 otherwise)
- [ ] Task 2.2: Add admin endpoints in `crates/handlers/src/admin.rs`:
  - `GET /admin/users` — list all users (with blocked status)
  - `PATCH /admin/users/{id}/block` — toggle blocked (body: `{blocked: bool}`)
  - `GET /admin/disputes` — list all disputes
  - `PATCH /admin/disputes/{id}/resolve` — resolve/dismiss dispute (body: `{status, resolution}`)
- [ ] Task 2.3: Add `POST /orders/{id}/dispute` to `crates/handlers/src/orders.rs` — any auth user can create dispute on their order (body: `{reason}`)
- [ ] Task 2.4: Add blocked-user check to auth flow — in handlers that use `AuthUser`, after extracting user, return 403 if `blocked == true`. Add helper `get_active_user` in handlers.
- [ ] Task 2.5: Register admin routes in `crates/handlers/src/lib.rs` (`api_routes` + `api_routes_with_openapi`)
- [ ] Task 2.6: Add admin + dispute routes to `crates/worker/src/lib.rs` (worker router)
- [ ] Task 2.7: Write tests — unit tests for `require_admin` logic, integration tests for block/unblock + dispute CRUD + auth guard rejection

### Verification

- [ ] All admin endpoints return 403 for non-NodeOperator users
- [ ] Blocked users get 403 on authenticated endpoints
- [ ] Dispute lifecycle works: create → list → resolve
- [ ] `make check` passes

## Phase 3: Frontend + Docs

Add Users and Disputes tabs to operator console.

### Tasks

- [ ] Task 3.1: Add API client functions in `crates/frontend/src/api.rs` — `fetch_admin_users`, `toggle_user_blocked`, `fetch_admin_disputes`, `resolve_dispute`, `create_dispute`
- [ ] Task 3.2: Add "Users" tab to `crates/frontend/src/pages/operator.rs` — table of users with name, email, role, blocked badge, block/unblock toggle button
- [ ] Task 3.3: Add "Disputes" tab to `crates/frontend/src/pages/operator.rs` — table of disputes with order ID, reason, status badge, resolve/dismiss actions with resolution text input
- [ ] Task 3.4: Update CLAUDE.md — add new API endpoints to endpoint table, update migration table
- [ ] Task 3.5: Remove dead code, verify `make check` passes

### Verification

- [ ] Operator console shows Users and Disputes tabs
- [ ] Block/unblock toggle works from UI
- [ ] Dispute resolution works from UI
- [ ] `make check` passes (clippy + fmt + tests)

## Final Verification

- [ ] All acceptance criteria from spec met
- [ ] Tests pass (`make test`)
- [ ] Linter clean (`make clippy`)
- [ ] Build succeeds (`cargo build`)
- [ ] Documentation up to date (CLAUDE.md, API endpoint table)

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

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
