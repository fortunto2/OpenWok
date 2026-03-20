# Specification: Courier PWA & Mobile Optimization

**Track ID:** courier-pwa_20260321
**Type:** Feature
**Created:** 2026-03-21
**Status:** Draft

## Summary

The courier dispatch backend and basic desktop UI are complete (courier-dispatch_20260320 track). However, the 20-40 pilot couriers will use their phones, not desktops. The current UI has no PWA support (no manifest, no service worker, no install prompt) and the courier pages aren't optimized for mobile workflows.

This track adds PWA infrastructure (manifest, service worker, icons) and mobile-optimizes the courier experience: large touch targets, prominent availability toggle, active delivery focus view, and "Add to Home Screen" install prompt. Scoped to what's needed for the LA pilot — no push notifications, no GPS, no offline order creation.

## Acceptance Criteria

- [ ] PWA manifest.json with app name, icons (192px + 512px), theme color, display: standalone
- [ ] Service worker caches static assets (WASM bundle, CSS, icons) for fast reload
- [ ] index.html links manifest + registers service worker
- [ ] `/my-deliveries` page is mobile-optimized: full-width cards, 48px+ touch targets, active delivery prominently displayed
- [ ] Availability toggle is prominent on courier dashboard (large switch, not hidden in text)
- [ ] `/register-courier` form is mobile-friendly (proper input sizing, keyboard-appropriate types)
- [ ] "Add to Home Screen" install prompt shown to courier users (via `beforeinstallprompt` API)
- [ ] App is installable as PWA from Chrome/Safari on mobile (passes Lighthouse PWA checks for basic criteria)
- [ ] Existing 107 tests still pass, `make check` clean
- [ ] Deployed to production and verified on mobile viewport

## Dependencies

- Courier dispatch backend — complete (courier-dispatch_20260320)
- Auth (Supabase Google OAuth) — complete
- Dioxus frontend SPA — complete (14 routes)
- Cloudflare Workers deploy — complete (wrangler.toml + public/ assets)

## Out of Scope

- Push notifications (use SMS/email for MVP, or courier checks dashboard)
- GPS/geolocation tracking
- Offline order creation or status updates (read-only offline cache only)
- Native iOS/Android apps
- Multi-order batching / route optimization
- Courier earnings dashboard
- App Store / Play Store submission

## Technical Notes

- PWA manifest.json and service worker go in `crates/frontend/public/` (copied to `public/` at build time via `make build-frontend`)
- Wrangler `[assets]` serves `public/` with `not_found_handling = "single-page-application"` — service worker registration will work automatically
- Existing CSS in `crates/frontend/assets/style.css` has basic `@media (max-width: 640px)` — extend for courier-specific mobile layout
- Courier pages: `crates/frontend/src/pages/courier.rs` (RegisterCourier + MyDeliveries components)
- Availability toggle needs `PATCH /api/couriers/{id}/available` — endpoint already exists
- WebSocket for real-time order updates already exists (`/api/ws/orders/{id}`)
- Icons can be generated from the OpenWok logo (orange theme, `#e85d04`)
- Tailwind CSS is configured (`input.css` + `@tailwindcss/cli`) but courier pages use vanilla CSS classes — stay consistent with existing `style.css` approach
