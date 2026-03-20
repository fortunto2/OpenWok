# OpenWok Phase 2 — Dioxus Frontend + WebSocket

**Status:** [x] Complete
**Track:** phase2-frontend

## Context Handoff

**Intent:** Build customer-facing web app and node operator console using Dioxus fullstack. Real-time order tracking via WebSocket.

**What's done:** Rust core (pricing, orders, state machine) + axum API (REST CRUD for restaurants, orders, couriers). 40 tests.

**Key files:** `crates/core/`, `crates/api/`, `CLAUDE.md`, `docs/mvp-deck.pdf`

---

- [x] Task 1.1: Add `crates/frontend/` Dioxus project. <!-- sha:e99c1b3 --> Cargo.toml with `dioxus = { features = ["fullstack", "router"] }`. Basic App component with router: `/` (home), `/restaurants` (list), `/restaurant/:id` (menu), `/order/:id` (tracking), `/operator` (console).
- [x] Task 1.2: Restaurant listing page <!-- sha:5a9f9eb --> — fetch `GET /restaurants` from API, display cards with name, zone, menu item count. Responsive grid layout.
- [x] Task 1.3: Restaurant menu page <!-- sha:26dc382 --> — fetch `GET /restaurants/:id`, show menu items with prices. "Add to cart" button per item. Cart component with running total.
- [x] Task 1.4: Order placement page <!-- sha:bbabe0f --> — cart summary, delivery address input, pricing breakdown preview (6 lines: Food / Delivery / Tip / Federal $1 / Local Ops / Processing). "Place Order" button → `POST /orders`.
- [x] Task 1.5: Add WebSocket support to axum API <!-- sha:5488f02 --> — `ws://host/ws/orders/:id` for real-time status updates. Broadcast `OrderStatusChanged` events. Use tokio broadcast channel.
- [x] Task 1.6: Order tracking page <!-- sha:0faf603 --> — show order status with timeline (Created → Confirmed → Preparing → Ready → InDelivery → Delivered). Live updates via WebSocket. Open-book receipt always visible.
- [x] Task 1.7: Node operator console <!-- sha:edb795c --> — `/operator` page. Dashboard: active orders, restaurants count, couriers online, today's revenue breakdown (federal vs local). List of recent orders with status.
- [x] Task 1.8: Courier assignment <!-- sha:1f7fca4 --> — operator can assign available couriers to orders. `PATCH /orders/:id/courier` endpoint. Dropdown of available couriers in zone.
- [x] Task 1.9: Responsive CSS/styling <!-- sha:a95a31c --> — clean minimal design. Mobile-first. Use Dioxus class-based styling or Tailwind via CDN.
- [x] Task 1.10: Run `make check`. Commit. <!-- sha:see-below -->
