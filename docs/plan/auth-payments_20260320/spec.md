# Specification: Auth + Stripe Payments

**Track ID:** auth-payments_20260320
**Type:** Feature
**Created:** 2026-03-20
**Status:** Draft

## Summary

Add user authentication (Supabase Auth with Google OAuth) and real payment processing (Stripe Connect with split payments) to OpenWok. This is PRD Phase 6 — the last major backend feature before pilot readiness.

Users authenticate via Google OAuth through Supabase. The backend verifies Supabase JWT tokens. Orders are linked to authenticated users. Checkout creates a Stripe Checkout Session with split payments: restaurant gets food revenue, courier gets delivery + tip, platform gets $1 federal fee + local ops fee. Stripe webhooks confirm payment and transition order status.

## Acceptance Criteria

- [x] JWT verification: backend validates Supabase JWT tokens (Authorization: Bearer header)
- [x] User CRUD: create user on first login, get user by Supabase ID, link orders to users
- [x] Protected routes: order creation, courier management require auth; restaurants list, health, economics remain public
- [x] Stripe Checkout Session: created on order placement with correct split amounts from pricing calculator
- [x] Stripe webhook: `checkout.session.completed` updates order status to Confirmed, `payment_intent.payment_failed` cancels order
- [x] Payment tracking: Payment record with status (Pending/Succeeded/Failed/Refunded) linked to order
- [x] Frontend login: "Sign in with Google" button redirects to Supabase OAuth, callback stores JWT
- [x] Frontend checkout: redirects to Stripe Checkout page, returns to order tracking on success
- [x] Worker parity: auth + payment routes work in Cloudflare Worker with D1
- [x] Tests: unit tests for JWT verification, payment creation; integration test for order→payment flow
- [x] `make check` passes (clippy clean, fmt clean, all tests pass)

## Dependencies

- `jsonwebtoken` crate (JWT verification)
- `reqwest` (Stripe API calls from api crate, already in workspace)
- Supabase project with Google OAuth configured (external)
- Stripe account with Connect enabled (external, test mode for dev)
- Wrangler secrets for production: SUPABASE_JWT_SECRET, STRIPE_SECRET_KEY, STRIPE_WEBHOOK_SECRET

## Out of Scope

- Restaurant onboarding / Connected Account creation (Phase 7)
- Courier PWA (Phase 7)
- Refund flow (post-pilot)
- Email/password auth (Google OAuth only for MVP)
- Stripe Elements / embedded payment form (using Stripe Checkout redirect)
- Multi-node auth isolation (Phase 8)

## Technical Notes

### Auth Architecture
- Frontend: redirect to Supabase Auth URL (Google OAuth) → callback captures JWT → stored in localStorage
- Backend: `jsonwebtoken` crate verifies JWT signature with Supabase JWT secret
- `AuthUser` extractor in handlers — generic, works in both axum (api) and manually in worker
- Supabase JWT claims: `sub` (user UUID), `email`, `role`, `exp`

### Payment Architecture
- Stripe Checkout Sessions (not raw PaymentIntents) — simplest redirect-based flow
- Split via `line_items` + `payment_intent_data.transfer_data` for restaurant Connected Account
- Platform keeps federal + local ops fees (default Stripe behavior for non-transferred amounts)
- Webhook signature verification: HMAC-SHA256 (manual impl, no stripe-rust needed)

### Worker Constraints
- D1Database is !Send → can't impl Repository trait → D1Repo has matching methods
- `worker::Fetch` for Stripe API calls (can't use reqwest in Workers)
- JWT verification works fine (jsonwebtoken is pure Rust, no I/O)

### Existing Patterns to Follow
- Repository trait + SqliteRepo/D1Repo for data access
- Generic handlers `<R: Repository>` in handlers crate
- Separate worker routes mirroring handler logic
- utoipa annotations for OpenAPI docs
- D1-compatible SQL migrations
