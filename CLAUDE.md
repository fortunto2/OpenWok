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
- **Runtime:** Single Cloudflare Worker = API (worker-rs, `/api/*`) + static assets (Dioxus WASM SPA, `/*`)
- **Database:** SQLite (rusqlite) locally, Cloudflare D1 in production — migrated from in-memory HashMap
- **Frontend:** Dioxus 0.6 web SPA (WASM) — migrating from fullstack mode (remove `#[server]`, use reqwest directly)
- **Payments:** Stripe Connect (split payments: restaurant + courier + federal + local)
- **Auth:** Supabase Auth (Google OAuth)
- **Geo:** zone-based (no PostGIS for MVP)
- **Deploy:** `dx build --platform web --release` → `wrangler deploy` (one Worker, one domain, zero CORS)

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
  api/      — openwok-api: axum REST server with SQLite (D1-compatible)
migrations/ — D1-compatible SQL migrations (shared with rusqlite)
```

## Run Commands

```bash
cargo run -p openwok-api       # Start server on http://localhost:3000/api
DATABASE_PATH=data/openwok.db cargo run -p openwok-api  # With custom DB path
make test                       # Run all tests
make clippy                     # Lint
make fmt                        # Check formatting
make check                      # test + clippy + fmt
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | /api/health | Health check |
| GET | /api/restaurants | List restaurants |
| GET | /api/restaurants/{id} | Get restaurant |
| POST | /api/restaurants | Create restaurant |
| POST | /api/orders | Create order (returns pricing breakdown) |
| GET | /api/orders/{id} | Get order |
| PATCH | /api/orders/{id}/status | Transition order status |
| POST | /api/orders/{id}/assign | Assign courier to order |
| GET | /api/couriers | List available couriers |
| POST | /api/couriers | Register courier |
| PATCH | /api/couriers/{id}/available | Toggle availability |
| WS | /api/ws/orders/{id} | Real-time order status updates |

## Key Documents

- `docs/prd.md` — PRD: vision, phases, what's built, what's next
- `docs/mvp-deck.pdf` — full MVP concept deck
- `docs/workflow.md` — TDD policy, commit strategy, verification checkpoints
- `planning/ROADMAP.md` — 12-month roadmap with decision gates
- `docs/plan-done/` — completed phase specs (mvp-core, phase2-frontend, phase3-payments, phase4-federation)

## Architecture Standards

**Layered architecture (dependencies point inward):**
```
[Domain types]  ←  [Use cases / services]  ←  [API handlers]  ←  [Framework (axum)]
   crates/core         crates/core              crates/api           crates/api
```
- Core has ZERO framework deps — pure Rust types + logic
- API depends on Core, never the reverse
- New crates (payments, federation) depend on Core, not on API

**Async patterns:**
- All I/O is async (tokio) — DB queries, HTTP, WebSocket
- Use `tokio::spawn` for background jobs (dispatch, notifications), not blocking threads
- Channels (`mpsc`, `broadcast`) for event propagation — not shared mutable state
- WebSocket: `broadcast::Sender<OrderEvent>` for real-time updates (already exists)

**Event sourcing (for orders):**
```rust
// Every state change = append-only event, projections for reads
pub trait EventStore {
    async fn append(&self, stream_id: &str, events: &[DomainEvent]) -> Result<()>;
    async fn load(&self, stream_id: &str) -> Result<Vec<DomainEvent>>;
}
// Order state = fold over events, never mutate directly
```

**Error handling:**
- Domain errors: typed enums via `thiserror` (OrderError, PaymentError, DispatchError)
- API errors: map domain errors → HTTP status + JSON body
- Never `unwrap()` in production code — `?` or explicit error handling
- Never `panic!` — use `tracing::error!` + return error

**Database (when migrating from HashMap):**
- sqlx with compile-time query checking where possible
- Migrations in `migrations/` directory
- Connection pool via `sqlx::PgPool` in AppState
- Transactions for multi-table operations (order + items + pricing)

**Type safety:**
- Newtype IDs: `RestaurantId(Uuid)`, `OrderId(Uuid)` — no String IDs
- Money: `Money` newtype over `Decimal` — already done, never use f64 for money
- Enums for statuses: `OrderStatus`, `CourierKind` — not strings
- All public types: `Clone + Debug + Serialize + Deserialize`

**Testing:**
- Unit tests in each module (`#[cfg(test)]`)
- Integration tests in `tests/` for API endpoints
- Use `axum::test::TestClient` or direct handler calls for API tests
- Fixture data in `tests/fixtures/` (JSON files for realistic test data)

## Do

- TDD for all business logic
- Read docs/mvp-deck.pdf for full context
- Keep pricing calculator as the core innovation (6-line receipt)
- Event log for all order state changes (append-only, never delete)
- Tracing spans on every API handler and service method
