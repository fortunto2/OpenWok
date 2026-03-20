# OpenWok Phase 2 — Dioxus Frontend + WebSocket

**Status:** [ ] Not Started
**Track:** phase2-frontend

## Context Handoff

**Intent:** Build customer-facing web app and node operator console using Dioxus fullstack. Real-time order tracking via WebSocket.

**What's done:** Rust core (pricing, orders, state machine) + axum API (REST CRUD for restaurants, orders, couriers). 40 tests.

**Key files:** `crates/core/`, `crates/api/`, `CLAUDE.md`, `docs/mvp-deck.pdf`

---

- [ ] Task 1.1: Add `crates/frontend/` Dioxus project. Cargo.toml with `dioxus = { features = ["fullstack", "router"] }`. Basic App component with router: `/` (home), `/restaurants` (list), `/restaurant/:id` (menu), `/order/:id` (tracking), `/operator` (console).
- [ ] Task 1.2: Restaurant listing page — fetch `GET /restaurants` from API, display cards with name, zone, menu item count. Responsive grid layout.
- [ ] Task 1.3: Restaurant menu page — fetch `GET /restaurants/:id`, show menu items with prices. "Add to cart" button per item. Cart component with running total.
- [ ] Task 1.4: Order placement page — cart summary, delivery address input, pricing breakdown preview (6 lines: Food / Delivery / Tip / Federal $1 / Local Ops / Processing). "Place Order" button → `POST /orders`.
- [ ] Task 1.5: Add WebSocket support to axum API — `ws://host/ws/orders/:id` for real-time status updates. Broadcast `OrderStatusChanged` events. Use tokio broadcast channel.
- [ ] Task 1.6: Order tracking page — show order status with timeline (Created → Confirmed → Preparing → Ready → InDelivery → Delivered). Live updates via WebSocket. Open-book receipt always visible.
- [ ] Task 1.7: Node operator console — `/operator` page. Dashboard: active orders, restaurants count, couriers online, today's revenue breakdown (federal vs local). List of recent orders with status.
- [ ] Task 1.8: Courier assignment — operator can assign available couriers to orders. `PATCH /orders/:id/courier` endpoint. Dropdown of available couriers in zone.
- [ ] Task 1.9: Responsive CSS/styling — clean minimal design. Mobile-first. Use Dioxus class-based styling or Tailwind via CDN.
- [ ] Task 1.10: Run `make check`. Commit.
