# Implementation Plan: Pilot Infrastructure (LA Node)

**Track ID:** pilot-infra_20260320
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-20
**Status:** [ ] Not Started

## Overview

Add pilot-critical infrastructure that doesn't depend on auth: realistic LA restaurant data, public economics transparency page, admin metrics for KPIs, PostHog analytics for funnel tracking, and frontend error boundary. All work builds on existing SQLite schema and axum API.

## Phase 1: Data Foundation <!-- checkpoint:db94c16 -->
Expand seed data and add metrics columns to orders table.

### Tasks
- [x] Task 1.1: Create `migrations/0003_pilot_zones.sql` <!-- sha:8846d1f --> — add 4 new zones (Venice, Santa Monica, Koreatown, Silver Lake) to existing Downtown LA + Hollywood.
- [x] Task 1.2: Create `migrations/0004_seed_pilot_restaurants.sql` <!-- sha:fec25b8 --> — seed 15 LA restaurants across 6 zones with 80+ realistic menu items (real names, market prices). Update `crates/api/src/db.rs` to apply new migrations.
- [x] Task 1.3: Add `estimated_eta` <!-- sha:db94c16 --> (INTEGER, nullable) and `actual_delivery_at` (TEXT, nullable) columns to orders table via `migrations/0005_order_metrics.sql`. Update `Order` type in `crates/core/src/order.rs` with new fields. Update `crates/api/src/routes/orders.rs` to read/write new columns.

### Verification
- [x] `make test` passes — seed data loads without conflicts
- [x] `SELECT count(*) FROM restaurants` returns 18+ (3 existing + 15 new)
- [x] `SELECT count(*) FROM zones` returns 6+

## Phase 2: Public Economics + Admin Metrics
API endpoints for transparency and pilot monitoring.

### Tasks
- [ ] Task 2.1: Add `GET /api/public/economics` endpoint in new file `crates/api/src/routes/economics.rs` — returns JSON with: total_orders, total_food_revenue, total_delivery_fees, total_federal_fees, total_local_ops_fees, total_processing_fees, avg_order_value. Add `Cache-Control: public, max-age=300` header. Register route in `crates/api/src/main.rs`.
- [ ] Task 2.2: Add `GET /api/admin/metrics` endpoint in new file `crates/api/src/routes/metrics.rs` — returns JSON with: order_count, orders_by_status, on_time_delivery_rate (orders where actual_delivery_at - created_at < estimated_eta), avg_eta_error_minutes, revenue_breakdown (same as economics), courier_utilization (available/total), orders_by_zone. Register route in `crates/api/src/main.rs`.
- [ ] Task 2.3: Write integration tests for both endpoints in `crates/api/src/routes/economics.rs` and `crates/api/src/routes/metrics.rs` — test with seeded data, test empty DB edge case.

### Verification
- [ ] `curl /api/public/economics` returns valid JSON with all fields
- [ ] `curl /api/admin/metrics` returns valid JSON with KPI breakdown
- [ ] Integration tests pass

## Phase 3: Frontend Pages
Public economics page and admin metrics dashboard.

### Tasks
- [ ] Task 3.1: Add `/economics` route to `crates/frontend/src/main.rs` — PublicEconomicsPage component that fetches `/api/public/economics` and renders: hero section explaining open-book model, breakdown table (Food Revenue / Delivery Fees / Federal Fees / Local Ops / Processing), total orders count, avg order value. Add navigation link in header/footer.
- [ ] Task 3.2: Add metrics dashboard section to OperatorConsole in `crates/frontend/src/main.rs` — new "Metrics" tab that fetches `/api/admin/metrics` and renders: KPI cards (order count, on-time rate, avg ETA error), orders-by-zone breakdown, courier utilization bar.
- [ ] Task 3.3: Add PostHog JS snippet to frontend — create/update `crates/frontend/assets/index.html` (or Dioxus head config) with PostHog `<script>` tag. Add `posthog.capture()` calls via `web_sys` interop in key user actions: `restaurant_view` (RestaurantMenu mount), `add_to_cart` (add item click), `checkout_start` (Checkout mount), `order_placed` (successful order creation). Use env var `POSTHOG_API_KEY` with fallback to empty string (no-op when not configured).
- [ ] Task 3.4: Add `ErrorBoundary` component wrapping app root in `crates/frontend/src/main.rs` — on error, display user-friendly message and capture `frontend_error` event to PostHog with error details.

### Verification
- [ ] `/economics` page renders with aggregate data
- [ ] Operator console shows metrics tab with KPI cards
- [ ] PostHog events fire in browser console (verify with PostHog debug mode)
- [ ] Error boundary catches simulated error gracefully

## Phase 4: Deploy & Docs
Update documentation, clean up, verify everything works.

### Tasks
- [ ] Task 4.1: Update `CLAUDE.md` — add new API endpoints (`/api/public/economics`, `/api/admin/metrics`), new migrations (0003-0005), PostHog integration note, new frontend route (`/economics`).
- [ ] Task 4.2: Update `docs/prd.md` — mark Phase 7 pilot-infra items as partially complete, add reference to this track.
- [ ] Task 4.3: Run `make check` — ensure all tests pass, clippy clean, fmt clean. Fix any issues.

### Verification
- [ ] CLAUDE.md reflects current project state
- [ ] `make check` passes cleanly

## Final Verification
- [ ] All acceptance criteria from spec met
- [ ] Tests pass (`make check`)
- [ ] Linter clean (clippy zero warnings)
- [ ] Build succeeds
- [ ] Documentation up to date
- [ ] No regressions in existing functionality (order flow, operator console)

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
