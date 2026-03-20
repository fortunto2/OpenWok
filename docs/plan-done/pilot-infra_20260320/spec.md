# Specification: Pilot Infrastructure (LA Node)

**Track ID:** pilot-infra_20260320
**Type:** Feature
**Created:** 2026-03-20
**Status:** Draft

## Summary

Prepare OpenWok for a controlled LA pilot by adding the data, metrics, analytics, and public transparency features that don't require auth (Phase 6 dependency). This track delivers: realistic restaurant seed data (10-20 LA restaurants across 4-6 zones), a public economics page (open-book aggregate financials — the core brand differentiator), admin metrics endpoints for pilot KPIs, PostHog analytics for funnel tracking, and basic error monitoring via Cloudflare Analytics + frontend error boundary.

Auth-dependent items (restaurant onboarding flow, courier PWA, user blocking, dispute resolution) are deferred to a separate `pilot-onboarding` track after Phase 6 (Auth + Payments).

## Acceptance Criteria

- [x] Migration `0003_seed_pilot_restaurants.sql` seeds 15+ LA restaurants across 4+ zones with 80+ menu items
- [x] `GET /api/public/economics` returns aggregate financials (total orders, revenue breakdown, avg order value) — no PII
- [x] `GET /api/admin/metrics` returns pilot KPIs: order count, on-time rate, avg ETA error, revenue breakdown, courier utilization
- [x] `OrderMetrics` fields (`estimated_eta`, `actual_delivery_at`) added to orders table and domain types
- [x] Public economics frontend page at `/economics` renders aggregate data with breakdown chart
- [x] Admin metrics section added to operator console with KPI dashboard
- [x] PostHog JS snippet loaded in frontend HTML — tracks: `page_view`, `restaurant_view`, `add_to_cart`, `checkout_start`, `order_placed`
- [x] Frontend error boundary catches render errors and reports to PostHog as `frontend_error` event
- [x] All existing tests pass (`make check`) — no regressions
- [x] New endpoints have integration tests

## Dependencies

- Existing SQLite/D1 schema (migrations 0001-0002)
- PostHog account + project API key (EU instance per user preferences)
- No auth dependency — all new endpoints are either public or operator-console-level

## Out of Scope

- Restaurant onboarding flow (needs auth — separate track)
- Courier PWA (needs auth — separate track)
- User blocking / dispute resolution (needs auth — separate track)
- Stripe payments integration (Phase 6)
- Real-time WebSocket for metrics (polling sufficient for pilot)
- Custom PostHog dashboards (configured in PostHog UI, not in code)
- Sentry integration (Cloudflare Analytics sufficient for MVP pilot)

## Technical Notes

- **Seed data:** Use real LA restaurant names and realistic menu items with market prices. Zones: Downtown LA, Hollywood, Venice, Santa Monica, Koreatown, Silver Lake.
- **Public economics:** Aggregate query on orders table — `SUM(food_total)`, `SUM(federal_fee)`, etc. Endpoint is cacheable (add `Cache-Control: max-age=300`).
- **OrderMetrics:** Add `estimated_eta` (INTEGER, minutes) and `actual_delivery_at` (TEXT, ISO8601) columns to orders table. Calculate on-time rate as: orders where `actual - created < estimated_eta`.
- **PostHog:** Use PostHog JS snippet (`posthog-js`) loaded via `<script>` tag in `index.html` — no Rust dependency needed. WASM frontend calls `window.posthog.capture()` via `web_sys::js_sys` interop.
- **Error boundary:** Dioxus `ErrorBoundary` component wrapping the app root. On error, capture to PostHog.
- **Admin metrics:** Reuse pricing fields already in orders table. Group by time period (day/week) for trend data.
- **D1 compatibility:** All new SQL must be SQLite-compatible (no PostgreSQL features). Test with rusqlite locally.
