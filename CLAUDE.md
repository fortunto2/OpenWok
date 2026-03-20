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
- **Frontend:** Dioxus 0.6 web SPA (WASM) — uses reqwest for API calls, compiled with `dx build --platform web`
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
  core/      — openwok-core: domain types, pricing, order state machine, Repository trait
  handlers/  — openwok-handlers: shared axum route handlers generic over Repository
  api/       — openwok-api: axum REST server with SqliteRepo + WebSocket
  frontend/  — openwok-frontend: Dioxus web SPA (7 pages + operator console)
  worker/    — openwok-worker: Cloudflare Worker with D1Repo (standalone workspace)
migrations/  — D1-compatible SQL migrations (shared with rusqlite)
```

**Repository pattern:**
```
[Repository trait]  ←  [handlers crate]  ←  [api: SqliteRepo]
   crates/core          crates/handlers      crates/api
                                         ←  [worker: D1Repo]
                                             crates/worker
```
- `Repository` trait in core defines async data access methods
- `handlers` crate has axum handlers generic over `R: Repository`
- `api` uses SqliteRepo (implements Repository) + handlers crate
- `worker` uses D1Repo (same method signatures, can't impl trait due to !Send D1Database) + worker::Router

## Run Commands

```bash
cargo run -p openwok-api       # Start server on http://localhost:3000/api
DATABASE_PATH=data/openwok.db cargo run -p openwok-api  # With custom DB path
make test                       # Run all tests
make clippy                     # Lint
make fmt                        # Check formatting
make check                      # test + clippy + fmt
```

## Development Workflow

**Frontend (Dioxus SPA):**
```bash
cd crates/frontend && dx serve    # Dev server with hot-reload at http://localhost:8080
                                  # MUST run from crates/frontend/ (where Dioxus.toml is)
                                  # API proxied to http://localhost:3000/api
```

**Backend (local axum):**
```bash
cargo run -p openwok-api          # Local API server at http://localhost:3000
```

**Full stack local:**
```bash
# Terminal 1:
cargo run -p openwok-api
# Terminal 2:
cd crates/frontend && dx serve
# Open http://localhost:8080
```

**Production (Cloudflare Worker):**
```bash
make deploy                       # worker-build + wrangler deploy
# Live at https://openwok.superduperai.co
# API: /api/*    Frontend: /* (static SPA from public/)
```

**Frontend build for production:**
```bash
cd crates/frontend && dx build --platform web --release
cp -r dist/* ../worker/public/    # Copy WASM bundle to worker static assets
```

## Visual Testing (Playwright MCP)

When building or reviewing frontend changes:
1. Start dev server: `cd crates/frontend && dx serve` (background)
2. Use Playwright MCP to navigate to `http://localhost:8080`
3. Take screenshots of key pages: restaurants, menu, cart, checkout, order tracking
4. Check browser console for errors
5. Test at mobile viewport (375px) for responsive layout

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
| GET | /api/public/economics | Aggregate financials (public, cached 5min) |
| GET | /api/admin/metrics | Pilot KPIs (order count, on-time rate, revenue) |
| WS | /api/ws/orders/{id} | Real-time order status updates |

## OpenAPI / Swagger

Use `utoipa` for auto-generated OpenAPI docs from Rust code:
```toml
# crates/api or crates/worker Cargo.toml
utoipa = { version = "5", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "9", features = ["axum"] }
```

Annotate handlers with `#[utoipa::path(...)]`, serve Swagger UI at `/api/docs`.
See `libraries.yaml` → `openapi_codegen` → `utoipa` for details.

## Key Documents

- `docs/prd.md` — PRD: vision, phases, what's built, what's next
- `docs/mvp-deck.pdf` — full MVP concept deck
- `docs/workflow.md` — TDD policy, commit strategy, verification checkpoints
- `planning/ROADMAP.md` — 12-month roadmap with decision gates
- `docs/plan-done/` — completed phase specs (mvp-core, phase2-frontend, pilot-infra, cf-workers-deploy)
- `docs/plan/pilot-infra_20260320/` — completed: pilot infrastructure (data, metrics, economics, PostHog)
- `docs/evolution.md` — factory evolution log (cross-retro learnings)
- `docs/retro/` — pipeline retrospectives

## Migrations

| File | Description |
|------|-------------|
| `migrations/0001_init.sql` | Initial schema (zones, restaurants, menu_items, couriers, orders, order_items) |
| `migrations/0002_seed_la.sql` | Seed data: 2 zones, 3 restaurants, 9 menu items |
| `migrations/0003_pilot_zones.sql` | Add 4 new LA zones (Venice, Santa Monica, Koreatown, Silver Lake) |
| `migrations/0004_seed_pilot_restaurants.sql` | Seed 15 new restaurants with 80+ menu items across 6 zones |
| `migrations/0005_order_metrics.sql` | Add `estimated_eta` and `actual_delivery_at` columns to orders |

## Frontend Routes

| Path | Component | Description |
|------|-----------|-------------|
| `/` | Home | Landing page |
| `/restaurants` | RestaurantList | Browse restaurants |
| `/restaurant/:id` | RestaurantMenu | Menu + cart |
| `/checkout` | Checkout | Order with 6-line pricing |
| `/order/:id` | OrderTracking | Real-time status timeline |
| `/operator` | OperatorConsole | Node operator dashboard (Overview + Metrics tabs) |
| `/economics` | PublicEconomicsPage | Open-book economics transparency page |

## Analytics (PostHog)

PostHog JS snippet loaded in frontend. Events tracked: `restaurant_view`, `add_to_cart`, `checkout_start`, `order_placed`, `frontend_error`. Configure via `window.__POSTHOG_API_KEY__` (EU instance). No-op when key not set.

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

**Database (SQLite/D1):**
- rusqlite locally, Cloudflare D1 in production (both SQLite-compatible)
- Migrations in `migrations/` directory (D1-compatible SQL)
- Connection pool via `SqlitePool` wrapper in AppState
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
