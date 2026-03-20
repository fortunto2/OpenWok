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
- **API:** Cloudflare Workers (worker-rs with axum compatibility)
- **Database:** Cloudflare D1 (SQLite-based, per-node)
- **Frontend:** Dioxus static build → Cloudflare Pages (SPA)
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
Port from axum in-memory to CF Workers + D1.

**Why CF Workers:**
- Edge deployment (low latency for LA users)
- D1 = SQLite (simpler than PostgreSQL for MVP)
- Free tier generous for pilot (100K requests/day)
- wrangler CLI already configured locally

**Tasks:**
1. Create `wrangler.toml` with D1 binding, Workers config
2. Add `worker` crate dependency with axum feature to `crates/api/`
3. Create D1 schema migrations (restaurants, menu_items, orders, order_items, couriers, zones)
4. Replace in-memory `AppState` (HashMap) with D1 queries via `worker::d1`
5. Adapt `main.rs` entry point: `worker::event!` macro instead of `tokio::main`
6. Build Dioxus frontend as static SPA (`dx build --release`)
7. Create CF Pages config for static frontend
8. Deploy API Worker + Pages frontend
9. Seed LA test data (3 restaurants, 2 zones)
10. Verify end-to-end: browse restaurants → place order → track status

**Acceptance criteria:**
- [ ] `wrangler dev` starts API locally with D1
- [ ] All 11 API endpoints work against D1
- [ ] Frontend loads from CF Pages, calls API Worker
- [ ] Order flow works end-to-end (create → assign → deliver)
- [ ] `make check` passes (existing tests adapted)
- [ ] Deployed to CF Workers (API) + CF Pages (frontend)

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
3. Real restaurant data (10-20 LA restaurants, 1-2 zones)
4. Admin tools: block user, resolve dispute, view metrics
5. Public economics page (open-book: total orders, total fees, breakdown)
6. PostHog analytics integration (order funnel, ETA accuracy, repeat rate)
7. Error monitoring (Sentry or CF analytics)

**Acceptance criteria:**
- [ ] Restaurant can self-onboard and manage menu
- [ ] Courier can accept and complete deliveries via mobile
- [ ] 10+ restaurants with real menus seeded
- [ ] Admin can block/unblock and resolve disputes
- [ ] Public economics page shows real aggregated data
- [ ] PostHog tracks: page_view → restaurant_view → add_to_cart → checkout → order_placed → delivered

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

- **API:** Cloudflare Workers (wrangler deploy)
- **Frontend:** Cloudflare Pages (static SPA)
- **Database:** Cloudflare D1
- **Auth:** Supabase (external)
- **Payments:** Stripe Connect (external)
- **Analytics:** PostHog EU

## KPIs (Pilot)

- On-time delivery rate > 90%
- ETA accuracy < 10 min error
- Repeat order rate > 30% (weekly)
- Courier earnings > $15/hr
- Restaurant savings vs DoorDash/UberEats > 10%
- Dispute rate < 2%
- Unit net profit: break-even or better at pilot fee level
