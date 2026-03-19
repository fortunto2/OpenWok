# CLAUDE.md — OpenWok

Federated food delivery platform. $1 federal fee + local node operators. Open-book pricing.

## Concept (from docs/mvp-deck.pdf)

- Platform as federation (like US states): Federal Core sets protocol + baseline, Node Operators run local markets
- $1 Federal Stewardship Fee per order → protocol, security, brand
- Local Operations Fee → node's actual costs (support, disputes, ops)
- Processing → pass-through (Stripe 2.9% + $0.30), shown separately
- Restaurants get 100% food revenue, couriers get 100% delivery + tips
- MVP: 1 LA node, 10-20 restaurants, 1-2 zones, 60-day timeline
- Permissioned federation: nodes join after KYB + tech audit
- Future: sidewalk delivery robots as "delivery agents"

## Tech Stack

- **Core (Rust):** domain logic, order engine, pricing calculator, federation protocol
- **Server:** Rust (axum) — REST API + WebSocket for real-time
- **Frontend (v1):** Dioxus (Rust fullstack) — customer web app + node operator console
- **Frontend (v2, future):** Next.js with Rust core via uniffi/WASM
- **Database:** PostgreSQL (per-node) + event log
- **Payments:** Stripe Connect (split payments: restaurant + courier + federal + local)
- **Auth:** Supabase Auth or custom JWT
- **Geo:** PostGIS for zone management, ETA calculation

## Repo

GitHub: https://github.com/fortunto2/OpenWok

## Key Entities

- **Order:** food items, delivery address, zone, pricing breakdown (6 lines)
- **Restaurant:** menu, zone, acceptance status
- **Courier/DeliveryAgent:** type (human/robot), zone, availability
- **Node:** operator info, zones, local fee config, SLA metrics
- **FederalCore:** protocol version, baseline rules, $1 fee

## MVP Scope (60 days)

1. Single LA node: catalog, order, payment, delivery, support
2. Open-book receipt: Food / Delivery / Tip / Federal Fee / Local Ops Fee / Processing
3. 10-20 restaurants in 1-2 zones
4. Mixed delivery (self + couriers)
5. Metrics: on-time, ETA error, repeat rate, $/hour, restaurant savings
6. Federation-ready: operator console + KYB + baseline logging

## Don't

- Build multi-node federation protocol yet — MVP is single node
- Build robot integration — that's wave 2
- Over-engineer — focus on order flow + open-book pricing

## Workspace Structure

```
crates/
  core/     — openwok-core: domain types, pricing calculator, order state machine
  api/      — openwok-api: axum REST server with in-memory state
```

## Run Commands

```bash
cargo run -p openwok-api       # Start server on http://localhost:3000
make test                       # Run all tests
make clippy                     # Lint
make fmt                        # Check formatting
make check                      # test + clippy + fmt
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | /health | Health check |
| GET | /restaurants | List restaurants |
| GET | /restaurants/{id} | Get restaurant |
| POST | /restaurants | Create restaurant |
| POST | /orders | Create order (returns pricing breakdown) |
| GET | /orders/{id} | Get order |
| PATCH | /orders/{id}/status | Transition order status |
| POST | /orders/{id}/assign | Assign courier to order |
| GET | /couriers | List available couriers |
| POST | /couriers | Register courier |
| PATCH | /couriers/{id}/available | Toggle availability |

## Key Documents

- `docs/mvp-deck.pdf` — full MVP concept deck
- `docs/workflow.md` — TDD policy, commit strategy, verification checkpoints
- `docs/plan/mvp-core_20260320/` — implementation plan + spec
- `planning/ROADMAP.md` — project roadmap

## Do

- TDD for all business logic
- Read docs/mvp-deck.pdf for full context
- Keep pricing calculator as the core innovation (6-line receipt)
