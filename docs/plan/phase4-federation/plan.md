# OpenWok Phase 4 — Federation Protocol + Multi-Node

**Status:** [ ] Not Started
**Track:** phase4-federation

## Context Handoff

**Intent:** Build the federation layer — multiple nodes, governance protocol, KYB onboarding, SLA metrics. This is what makes OpenWok unique vs just another delivery app.

**From MVP deck:**
- Permissioned federation: nodes join after KYB + tech audit
- Node = юрлицо, manages local market (zones, couriers, support)
- Federal Core = protocol version, baseline rules, $1 fee
- PolicyProposed → PolicyActivated governance events
- Local fee review cycle based on public cost aggregates
- Node types: cooperative, franchise, restaurant association

---

- [ ] Task 1.1: Node registration API — `POST /federation/nodes` with KYB data (legal name, tax id, zones, contact). Status: Pending → Approved → Active. Admin approval endpoint.
- [ ] Task 1.2: Multi-node data isolation — each node sees only its restaurants, orders, couriers. API middleware checks `X-Node-Id` header. Shared federal metrics endpoint.
- [ ] Task 1.3: Node operator console enhancements — local fee configuration (within baseline bounds), zone management (add/remove zones), courier onboarding.
- [ ] Task 1.4: Federation metrics API — `GET /federation/metrics` returns aggregate: total orders, avg delivery time, avg local fee, node count. No private data exposed.
- [ ] Task 1.5: SLA monitoring — track per-node: on-time %, ETA error, customer complaints, refund rate. Alert if below baseline threshold.
- [ ] Task 1.6: Governance events — event log table. `PolicyProposed(node, change_type, details)` → admin review → `PolicyActivated`. Immutable audit trail.
- [ ] Task 1.7: Baseline rules engine — configurable rules: max local fee, min courier pay, max ETA, required response time. Nodes must comply. API validates on order creation.
- [ ] Task 1.8: Node health dashboard — `/federation/nodes/:id/health`. Uptime, order volume, SLA score, local fee history.
- [ ] Task 1.9: API documentation — generate OpenAPI spec from axum routes (utoipa). Serve Swagger UI at `/docs`.
- [ ] Task 1.10: README.md update — architecture diagram, API endpoints, federation protocol description, setup guide. `make check`. Tag v0.1.0.
