# Specification: OpenWok MVP Core

**Track ID:** mvp-core_20260320
**Type:** Feature
**Created:** 2026-03-20
**Status:** Draft

## Summary

Build the foundational Rust codebase for OpenWok — a federated food delivery platform. This track covers the Cargo workspace setup, domain types, the open-book pricing calculator (6-line receipt), order state machine, and a REST API via axum. Single LA node scope.

The pricing calculator is the core innovation: every receipt shows exactly 6 lines (Food, Delivery, Tip, Federal Fee, Local Ops Fee, Processing) with zero hidden markup. Restaurants get 100% food revenue, couriers get 100% delivery + tips.

## Acceptance Criteria

- [ ] Cargo workspace compiles, `make check` passes (test + clippy + fmt)
- [ ] Domain types: `Money`, `Order`, `Restaurant`, `MenuItem`, `Courier`, `DeliveryAgent`, `Node`, `Zone`, `PricingBreakdown`
- [ ] Pricing calculator produces 6-line breakdown: Food / Delivery / Tip / Federal Fee ($1.00) / Local Ops Fee / Processing (2.9% + $0.30)
- [ ] Pricing calculator tested: edge cases (zero tip, zero delivery, large orders, rounding)
- [ ] Order state machine: Created → Confirmed → Preparing → ReadyForPickup → InDelivery → Delivered (+ Cancelled)
- [ ] Invalid state transitions rejected with errors
- [ ] axum REST API: health, restaurants CRUD, order create/status/list, courier availability
- [ ] API returns `PricingBreakdown` in order responses
- [ ] Integration tests for order flow (create → confirm → deliver)
- [ ] CLI smoke test: can create order and see pricing breakdown

## Dependencies

- Rust toolchain (stable)
- axum (web framework)
- serde / serde_json (serialization)
- uuid (IDs)
- chrono (timestamps)
- tokio (async runtime)
- rust_decimal (precise money math — no floats for currency)

## Out of Scope

- Database persistence (in-memory for now — PostgreSQL in next track)
- Stripe Connect integration
- Dioxus frontend
- Authentication / authorization
- Multi-node federation protocol
- Robot delivery agents
- PostGIS / real geo calculations
- WebSocket real-time updates

## Technical Notes

- Use `rust_decimal::Decimal` for all money — never f64 for currency
- Federal fee is hardcoded $1.00 (from deck + workflow.md invariant)
- Processing fee formula: `subtotal * 0.029 + 0.30` (Stripe pass-through)
- Local ops fee is configurable per Node (MVP: single value for LA node)
- Order IDs use UUID v4
- State machine uses enum + transition validation (no illegal states at type level where possible)
- Workspace structure: `crates/core` (domain + pricing), `crates/api` (axum server)
