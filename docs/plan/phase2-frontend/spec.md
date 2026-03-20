# Phase 2 — Acceptance Criteria

- [x] Dioxus frontend compiles and serves — `cargo build --workspace` succeeds, crates/frontend/src/main.rs
- [x] Restaurant list and menu pages work — RestaurantList + RestaurantMenu components
- [x] Cart + order placement with pricing breakdown — CartPanel + Checkout with 6-line receipt
- [x] WebSocket real-time order tracking — crates/api/src/routes/ws.rs + OrderTracking component
- [x] Node operator console with dashboard — OperatorConsole with stats grid + order list
- [x] Courier assignment from operator console — assign_courier server fn + OrderRow component
- [x] Mobile-responsive layout — @media (max-width: 640px) in assets/style.css
- [x] `make check` passes — 40 tests pass, clippy clean, fmt clean
