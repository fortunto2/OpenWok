# OpenWok Phase 3 — Stripe Payments + PostgreSQL

**Status:** [ ] Not Started
**Track:** phase3-payments

## Context Handoff

**Intent:** Replace in-memory storage with PostgreSQL. Integrate Stripe Connect for split payments (restaurant + courier + federal + local ops). Implement the open-book receipt as real payment flow.

**What's done:** Core + API + Frontend (Dioxus). In-memory HashMap storage.

**Key decisions from MVP deck:**
- Stripe Connect: platform creates connected accounts for restaurants + couriers
- $1 Federal Fee → platform account
- Local Ops Fee → node operator account
- Processing shown separately (Stripe's 2.9% + $0.30)
- Restaurants get 100% food, couriers get 100% delivery + tips

---

- [ ] Task 1.1: Add PostgreSQL via sqlx. Create `crates/db/` with migrations: `restaurants`, `couriers`, `zones`, `nodes`, `orders`, `order_items`. Use sqlx migrate. Connection pool in AppState.
- [ ] Task 1.2: Replace HashMap storage → sqlx queries. CRUD for restaurants, orders, couriers. Transaction for order creation (insert order + items + pricing in one tx).
- [ ] Task 1.3: Stripe Connect setup — add `stripe-rust` crate. Environment config: `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`. Create `crates/payments/` module.
- [ ] Task 1.4: Implement payment intent creation on order placement. Split payment: `transfer_data` to restaurant connected account (food amount), separate transfer to courier (delivery + tip). Platform keeps $1 federal fee.
- [ ] Task 1.5: Stripe webhook handler — `/webhooks/stripe`. Handle `payment_intent.succeeded` → update order status to Confirmed. Handle `payment_intent.failed` → cancel order.
- [ ] Task 1.6: Receipt endpoint — `GET /orders/:id/receipt` returns the 6-line open-book breakdown with Stripe payment confirmation. PDF generation optional.
- [ ] Task 1.7: Seed data migration — 3 LA restaurants with real-ish menus, 2 zones (Downtown LA, Hollywood), 1 node operator, 5 couriers.
- [ ] Task 1.8: Environment config — `.env` file with DATABASE_URL, STRIPE keys, API port. dotenv loading in main.rs.
- [ ] Task 1.9: Run `make check`. Integration test: create order → mock Stripe → verify payment split. Commit.
