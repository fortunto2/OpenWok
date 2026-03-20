# OpenWok — Full Roadmap

> LA node → federation → robots. $1 federal + local ops. Trusted invites.

## 0) Product Rules (1–3 days)

- [x] "Money Constitution" (6-line receipt): Food / Delivery / Tip / Federal $1 / Local Ops / Processing
- [x] Local Ops Fee formula for MVP (fixed per zone)
- [ ] Minimum SLA metrics: on-time %, ETA error, cancel rate, refund rate, time-to-resolution
- [ ] Prohibited items policy + incident flow (kill switch, freeze payouts)

## 1) Event Protocol v0 (3–5 days)

- [ ] Event schemas (JSON): OrderCreated / OrderAccepted / CourierAssigned / PickupConfirmed / DropoffConfirmed
- [ ] RefundRequested / RefundResolved
- [ ] PolicyProposed / PolicyActivated
- [ ] InviteCreated / InviteAccepted (trusted invites)
- [ ] Identifiers, idempotency, deduplication
- [ ] Event signing (server signing), schema versioning

## 2) Core Backend (LA node) — Rust (2–4 weeks)

### Service Framework
- [x] HTTP API (axum) + endpoints
- [ ] OpenAPI spec (utoipa)
- [ ] Auth: users/roles (customer/merchant/courier/operator/admin)
- [ ] Storage: Postgres (sqlx), migrations
- [ ] Job queue: Redis + background jobs (or NATS for event bus)
- [ ] Observability: logs + tracing + metrics (opentelemetry)

### Domains
- [x] Catalog: restaurants/menu/prices
- [x] Orders: cart → order → statuses (state machine)
- [x] Open-book pricing breakdown (6-line receipt)
- [ ] Tax/prep time in catalog
- [ ] Dispatch v1: rule-based (ETA, distance, fairness, load)
- [ ] Refunds/disputes: flow + evidence (photo/geo/timestamps)
- [ ] Event log (append-only table + projection tables)

## 3) Clients (2–6 weeks, parallel)

### Customer web (Dioxus SPA)
- [ ] Restaurant catalog with zone filter
- [ ] Menu + cart + open-book receipt preview
- [ ] Order placement + real-time tracking (WebSocket)
- [ ] Offline-first: local catalog cache, retry queue

### Courier app
- [ ] Accept order, navigation, checklist, status updates
- [ ] Incident buttons: "can't find", "restaurant delay", "customer unreachable"
- [ ] Store-and-forward on poor connectivity

### Operator console
- [ ] SLA dashboard, incidents, refunds, chargeback queue
- [ ] Local Ops Fee management (rules/limits/publish aggregates)
- [ ] Restaurant/courier onboarding (KYB/KYC workflow)
- [ ] Kill switch: disable merchant/courier, freeze payouts, block category

## 4) Payments (1–3 weeks)

- [ ] Payment flow: merchant of record decision
- [ ] Stripe Connect: split payments (restaurant + courier + federal + local)
- [ ] Processing as pass-through line in receipt
- [ ] Courier payouts: schedule, statuses, "never hold courier money" principle
- [ ] Basic anti-fraud: limits, velocity checks, suspicious patterns

## 5) Trusted Invites / Reputation (1–2 weeks)

- [ ] Invitation graph model: who can invite whom
- [ ] Influence rules: limits, depth=2, time-decay, penalties for bad invitee behavior
- [ ] Reputation tiers (trust tier separate from quality)
- [ ] Appeals and restoration (node procedures)

## 6) Federation (node #2) — after stable LA (2–6 weeks)

- [ ] Node Operator Agreement (approval process)
- [ ] Peering allowlist + tech audit
- [ ] Cross-node catalog (read-only) + reputation portability
- [ ] Event relay between nodes (server-to-server, tonic gRPC)
- [ ] CloudEvents format for inter-node events
- [ ] ed25519 event signatures
- [ ] Public network aggregates (no personal data)

## 7) Robots (wave 2) (pilot 6–12 weeks)

- [ ] DeliveryAgent type: HumanCourier / SidewalkRobot / DroneOperator
- [ ] Robot zones + exceptions (tele-ops interface)
- [ ] Robot metrics: utilization, exception rate, tele-ops cost
- [ ] Partner integration (Serve Robotics, Coco/DoorDash)
- [ ] Drones: only through Part 135 partner

## 8) LA GTM Checklist

- [ ] Pick 1–2 neighborhood pockets (Downtown LA, Hollywood)
- [ ] 10–20 restaurants, sign terms (open-book + fees)
- [ ] Recruit first couriers (20–40) + training
- [ ] Week 1: manual ops — collect data
- [ ] Weekly node report: SLA + cost aggregates → tune Local Ops Fee

## Rust Stack

| Component | Crate |
|-----------|-------|
| API | axum |
| DB | Postgres + sqlx |
| Cache/Jobs | Redis (later NATS/JetStream) |
| Auth | JWT + RBAC (scopes) |
| Telemetry | tracing + opentelemetry |
| Event log | append-only table + projection tables |
| Federation | tonic (gRPC) + cloudevents-sdk + ed25519-dalek |
| Frontend | Dioxus (SPA) |
