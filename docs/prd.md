---
type: prd
status: active
title: OpenWok — Federated Food Delivery Platform
created: 2026-03-20
stack: rust-cloudflare
deploy: cloudflare-workers + cloudflare-pages
---

# OpenWok — PRD

Federated food delivery: $1 federal fee + local node operators. Open-book pricing.

## Vision (from docs/mvp-deck.pdf)

Platform as federation (like US states):
- **Federal Core** = protocol + baseline rules + $1 stewardship fee per order
- **Node Operators** = local markets (cities/districts), set their own Local Ops Fee
- **Open-Book Receipt**: Food (100% → restaurant) / Delivery (100% → courier) / Tip (100% → courier) / Federal Fee ($1) / Local Ops Fee / Processing (Stripe pass-through)
- Restaurants get 100% food revenue, couriers get 100% delivery + tips
- MVP: 1 LA node, 10-20 restaurants, 1-2 zones

## Tech Stack

- **Core:** Rust — domain logic, pricing calculator, order state machine
- **Runtime:** Single Cloudflare Worker = API (worker-rs) + static assets (Dioxus WASM SPA)
- **Database:** Cloudflare D1 (SQLite-based, per-node)
- **Frontend:** Dioxus 0.6 web SPA (WASM), built with `dx build --platform web --release`, served as static assets from the same Worker
- **Payments:** Stripe Connect (split payments)
- **Auth:** Supabase Auth (Google OAuth)
- **Geo:** zone-based (no PostGIS needed for MVP)

## What's Built (Phases 1-2)

### Phase 1: MVP Core ✅
- Cargo workspace: `core`, `api`, `frontend`
- Domain types: Money (Decimal), Restaurant, MenuItem, Courier, Order, Zone, Node
- Pricing calculator: 6-line open-book receipt
- Order state machine: Created → Confirmed → Preparing → ReadyForPickup → InDelivery → Delivered (+ Cancelled)
- REST API (axum): 11 endpoints, in-memory HashMap
- 40 tests, clippy clean

### Phase 2: Dioxus Frontend ✅
- 6 routes: Home, RestaurantList, RestaurantMenu, Checkout, OrderTracking, OperatorConsole
- Cart state, place order flow, 6-line pricing breakdown at checkout
- WebSocket endpoint for order tracking
- Operator console: dashboard stats, assign courier, transition statuses
- Responsive CSS (mobile-first)

## What's Next

### Phase 5: Cloudflare Workers Migration
Port from axum in-memory to single CF Worker (API + static frontend) with D1.

**Architecture:** One Worker serves everything — API routes (`/api/*`) via worker-rs, all other paths via `[assets]` binding (Dioxus WASM SPA). Zero CORS, one domain, one deploy.

**Why CF Workers:**
- Edge deployment (low latency for LA users)
- D1 = SQLite (simpler than PostgreSQL for MVP)
- Free tier generous for pilot (100K requests/day)
- Single deployment unit (API + frontend together)
- wrangler 4.60.0 already configured locally

**Tasks:**
1. ~~Create `wrangler.toml`~~ ✅ Worker config + `[assets]` binding + D1 database binding
2. Add `worker` crate to `crates/api/`, adapt entry point from `tokio::main` to `worker::event!` macro (future: runtime swap)
3. ~~Create D1 schema migrations~~ ✅ `migrations/0001_init.sql` + `migrations/0002_seed_la.sql`
4. ~~Replace in-memory `AppState` (HashMap) with SQLite queries~~ ✅ rusqlite locally, D1-compatible schema
5. ~~Move all API routes under `/api/` prefix~~ ✅
6. Convert Dioxus frontend from fullstack to web SPA: remove `#[server]` functions, use reqwest directly (WASM-compatible), API_BASE = "/api" (same origin)
7. Optimize Dioxus.toml: `wasm_opt.level = "z"`, `pre_compress = true`, `index_on_404 = true`
8. ~~Add WASM release profile to workspace Cargo.toml~~ ✅ `opt-level = "z"`, `lto = true`, `panic = "abort"`, `strip = true`
9. Build pipeline: `dx build --platform web --release` → copy output to Worker's `public/` dir → `wrangler deploy`
10. ~~Seed LA test data (3 restaurants, 2 zones) via D1 migration~~ ✅
11. Verify end-to-end: browse restaurants → place order → track status on live URL

**Acceptance criteria:**
- [ ] `wrangler dev` starts locally: API on `/api/*`, SPA on `/*`
- [x] All 12 API endpoints work under `/api/` prefix against SQLite
- [ ] Dioxus SPA loads, all 6 routes work (client-side routing, 404 → index.html)
- [x] Order flow works end-to-end (create → assign → deliver) — integration test passes
- [ ] WASM bundle < 500KB (release + wasm-opt z)
- [x] `make check` passes (45 tests, clippy clean, fmt clean)
- [ ] Deployed: single `wrangler deploy`, live URL returns HTTP 200

### Phase 6: Auth + Stripe Payments
Real payments and user accounts.

**Tasks:**
1. Supabase Auth integration (Google OAuth, JWT verification in Worker)
2. Stripe Connect: create Connected Accounts for restaurants
3. Stripe PaymentIntent with transfer_data (split: restaurant + courier + federal + local)
4. Webhook endpoint `/webhooks/stripe` for payment confirmations
5. Order payment status tracking (pending → paid → refunded)
6. Receipt page with Stripe receipt link

**Acceptance criteria:**
- [ ] User can sign in with Google
- [ ] Test payment flow works in Stripe test mode
- [ ] Split payment reaches restaurant Connected Account
- [ ] Webhook updates order status on successful payment
- [ ] Refund flow works for cancelled orders

### Phase 7: Pilot Readiness (LA Node)
Production-ready for controlled pilot.

**Tasks:**
1. Restaurant onboarding flow (sign up → menu upload → go live)
2. Courier PWA (accept order, route, confirm delivery)
3. ~~Real restaurant data (10-20 LA restaurants, 1-2 zones)~~ **Done** — `pilot-infra_20260320`: 18 restaurants, 80+ items, 6 zones
4. ~~Admin tools: view metrics~~ **Done** — `GET /api/admin/metrics` + operator console Metrics tab. Block/dispute still needs auth.
5. ~~Public economics page (open-book: total orders, total fees, breakdown)~~ **Done** — `/economics` route + `GET /api/public/economics`
6. ~~PostHog analytics integration~~ **Done** — tracks: restaurant_view, add_to_cart, checkout_start, order_placed, frontend_error
7. ~~Error monitoring~~ **Done** — ErrorBoundary + PostHog `frontend_error` event

**Acceptance criteria:**
- [ ] Restaurant can self-onboard and manage menu
- [ ] Courier can accept and complete deliveries via mobile
- [x] 10+ restaurants with real menus seeded (18 restaurants, 6 zones)
- [ ] Admin can block/unblock and resolve disputes
- [x] Public economics page shows real aggregated data
- [x] PostHog tracks: page_view → restaurant_view → add_to_cart → checkout → order_placed → delivered

### Phase 8: Federation Protocol (post-pilot)
Multi-node support. Not in MVP scope.

- Node registration + KYB + approval flow
- Multi-node data isolation (X-Node-Id header middleware)
- Governance event log (PolicyProposed → PolicyActivated)
- Baseline rules engine (max local fee, min courier pay)
- Node-to-node sync (CloudEvents over HTTP)

## Open Decisions

### Gate A: Pricing Model
Monte Carlo simulation shows $1 fee is structurally unprofitable (-112% margin).
Recommended range: $3.35-$3.95 (base $3.60).
**Current code supports configurable fees** — pricing calculator takes federal_fee as parameter.
Decision: keep $1 for pilot to test market reaction, adjust based on pilot data.

### Gate B: Legal Role (MoR)
Who is merchant of record? Affects chargeback liability, restaurant contracts.
For MVP pilot: platform acts as marketplace (restaurants are MoR for food).

### Gate C: Brand
Using "OpenWok" for now. Domain/trademark TBD.

## Repo Structure

```
crates/
  core/       — openwok-core: domain types, pricing, order state machine
  api/        — openwok-api: REST API (currently axum, migrating to CF Workers)
  frontend/   — openwok-frontend: Dioxus SPA
docs/
  prd.md      — this file
  workflow.md — TDD policy, commit strategy
  plan-done/  — completed phase specs + plans
planning/
  ROADMAP.md  — 12-month roadmap with decision gates
  cost-simulation/    — Monte Carlo cost sim
  delivery-simulation/ — Delivery risk sim
programs/
  delivery-monte-carlo/   — Simulation code + reports
  service-cost-simulation/ — Cost analysis code
```

## Deploy

- **All-in-one:** Single Cloudflare Worker (`wrangler deploy`) — API routes + static SPA assets
- **Database:** Cloudflare D1 (SQLite, edge)
- **Auth:** Supabase (external)
- **Payments:** Stripe Connect (external)
- **Analytics:** PostHog EU
- **Build:** `dx build --platform web --release` → `wrangler deploy`

## KPIs (Pilot)

- On-time delivery rate > 90%
- ETA accuracy < 10 min error
- Repeat order rate > 30% (weekly)
- Courier earnings > $15/hr
- Restaurant savings vs DoorDash/UberEats > 10%
- Dispute rate < 2%
- Unit net profit: break-even or better at pilot fee level
