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
- **API:** Cloudflare Workers (worker-rs with axum compat) — migrating from standalone axum
- **Frontend:** Dioxus static SPA → Cloudflare Pages
- **Database:** Cloudflare D1 (SQLite) — migrating from in-memory HashMap
- **Payments:** Stripe Connect (split payments: restaurant + courier + federal + local)
- **Auth:** Supabase Auth (Google OAuth)
- **Geo:** zone-based (no PostGIS for MVP)
- **Deploy:** `wrangler deploy` (API Worker) + CF Pages (frontend)

## Federation Stack (Phase 4)

| Крейт | Зачем | Фаза |
|-------|-------|------|
| `tonic` + protobuf | Node-to-node RPC, .proto = спецификация протокола | Phase 4 |
| `cloudevents-sdk` | Стандартный формат событий (CNCF) — OrderCreated, PolicyActivated | Phase 3-4 |
| `ed25519-dalek` | Подпись событий нодой, верификация без доверия | Phase 4 |
| `cqrs-es` | Event sourcing для агрегатов (Order, Restaurant) — по необходимости | Future |
| `openraft` | Consensus для governance голосования — когда нод >3 | Future |
| `libp2p` (Kademlia) | Автодискавери нод — когда нод >10 | Future |

Паттерны из MVP deck: Matrix server-to-server + ActivityPub inbox/outbox + CloudEvents.

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

- Skip writing actual code — every task MUST produce new .rs files or modify existing ones
- Output <solo:done/> without new commits — if no code was written, it's NOT done
- Build robot integration — that's wave 2
- Over-engineer — but DO implement each plan task fully

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

- `docs/prd.md` — PRD: vision, phases, what's built, what's next
- `docs/mvp-deck.pdf` — full MVP concept deck
- `docs/workflow.md` — TDD policy, commit strategy, verification checkpoints
- `planning/ROADMAP.md` — 12-month roadmap with decision gates
- `docs/plan-done/` — completed phase specs (mvp-core, phase2-frontend, phase3-payments, phase4-federation)

## Do

- TDD for all business logic
- Read docs/mvp-deck.pdf for full context
- Keep pricing calculator as the core innovation (6-line receipt)
