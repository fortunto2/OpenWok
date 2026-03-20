# Implementation Plan: Courier PWA & Mobile Optimization

**Track ID:** courier-pwa_20260321
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-21
**Status:** [ ] Not Started

## Overview

Add PWA support (manifest, service worker, icons) and mobile-optimize the courier UI for the 20-40 pilot couriers who'll use phones. Three phases: PWA infrastructure, mobile UI, deploy + verify.

## Phase 1: PWA Infrastructure
Add manifest, service worker, and icons so the app is installable on mobile.

### Tasks
- [x] Task 1.1: Create `crates/frontend/public/manifest.json` <!-- sha:221528f --> — app name "OpenWok Courier", short_name "OpenWok", theme_color "#e85d04", background_color "#fafafa", display "standalone", start_url "/my-deliveries", icons array (192px + 512px)
- [x] Task 1.2: Create PWA icon files in `crates/frontend/public/` — generate `icon-192.png` and `icon-512.png` (solid orange circle with "OW" text, or use placeholder SVG-to-PNG). Also create `apple-touch-icon.png` (180px) <!-- sha:9b123be -->
- [x] Task 1.3: Create `crates/frontend/public/sw.js` — service worker that caches static assets (WASM bundle, CSS, manifest, icons) using cache-first strategy for assets, network-first for API calls <!-- sha:ec63b31 -->
- [x] Task 1.4: Update `crates/frontend/public/index.html` — add `<link rel="manifest">`, `<meta name="theme-color">`, `<meta name="apple-mobile-web-app-capable">`, `<link rel="apple-touch-icon">`, and inline script to register service worker <!-- sha:8962ece -->
- [ ] Task 1.5: Add install prompt component in `crates/frontend/src/pages/courier.rs` — detect `beforeinstallprompt` event via JS interop, show "Install OpenWok" banner on courier pages

### Verification
- [ ] `crates/frontend/public/manifest.json` is valid JSON and linked from index.html
- [ ] Service worker registers without console errors
- [ ] Chrome DevTools > Application shows manifest + service worker active

## Phase 2: Mobile-Optimized Courier UI
Redesign courier pages for thumb-friendly mobile use.

### Tasks
- [ ] Task 2.1: Redesign `MyDeliveries` component in `crates/frontend/src/pages/courier.rs` — add prominent availability toggle (large on/off switch using `PATCH /api/couriers/{id}/available`), active delivery card with large "Mark Delivered" button (min-height 48px), delivery history as compact cards
- [ ] Task 2.2: Add courier-specific mobile CSS to `crates/frontend/assets/style.css` — `.courier-toggle` (large switch), `.delivery-card-active` (full-width, prominent), `.delivery-action` (48px+ touch target), `.courier-header` (sticky status bar)
- [ ] Task 2.3: Improve `RegisterCourier` component in `crates/frontend/src/pages/courier.rs` — larger form inputs (min 44px height), `inputmode="text"` attributes, full-width submit button, better zone selector for touch
- [ ] Task 2.4: Add auto-refresh to `MyDeliveries` — poll `/api/my/deliveries` every 15s when courier is available (new orders appear without manual refresh), use `use_future` with interval

### Verification
- [ ] Courier pages render well at 375px width (iPhone SE)
- [ ] All interactive elements have min 44px touch target
- [ ] Availability toggle works (calls API, updates UI state)
- [ ] New deliveries appear within 15s without page refresh

## Phase 3: Deploy & Verify
Build, deploy, and verify on real mobile.

### Tasks
- [ ] Task 3.1: Run `make check` — all 107+ tests pass, clippy clean, fmt clean
- [ ] Task 3.2: Run `make deploy` — build frontend (with new PWA assets), build worker, wrangler deploy
- [ ] Task 3.3: Verify PWA on production — check manifest loads, service worker activates, app installable from mobile Chrome

### Verification
- [ ] Production site loads at openwok.superduperai.co
- [ ] `/my-deliveries` is mobile-optimized
- [ ] App installable as PWA (Lighthouse basic PWA check)

## Phase 4: Docs & Cleanup

### Tasks
- [ ] Task 4.1: Update CLAUDE.md with PWA-related files (manifest.json, sw.js, icons) and any new CSS classes
- [ ] Task 4.2: Remove dead code — unused imports, orphaned files, stale exports

### Verification
- [ ] CLAUDE.md reflects current project state
- [ ] Linter clean, tests pass

## Final Verification
- [ ] All acceptance criteria from spec met
- [ ] Tests pass
- [ ] Linter clean
- [ ] Build succeeds
- [ ] Documentation up to date
- [ ] PWA installable on mobile

## Context Handoff
_Summary for /build to load at session start — keeps context compact._

### Session Intent
Make the courier experience mobile-first and installable as a PWA for the LA pilot's 20-40 couriers.

### Key Files
- `crates/frontend/public/index.html` — add manifest link, SW registration, meta tags
- `crates/frontend/public/manifest.json` — NEW: PWA manifest
- `crates/frontend/public/sw.js` — NEW: service worker
- `crates/frontend/public/icon-192.png` — NEW: PWA icon
- `crates/frontend/public/icon-512.png` — NEW: PWA icon
- `crates/frontend/src/pages/courier.rs` — mobile-optimize MyDeliveries + RegisterCourier, add install prompt
- `crates/frontend/assets/style.css` — add courier-specific mobile CSS
- `wrangler.toml` — no changes needed (assets already served from public/)
- `Makefile` — no changes needed (build-frontend copies public/ already)

### Decisions Made
- PWA start_url = `/my-deliveries` (courier-centric, not home page)
- Cache-first for static assets, network-first for API (courier needs fresh data)
- No push notifications for MVP — courier polls/refreshes dashboard
- Use `beforeinstallprompt` for Chrome install banner, `apple-mobile-web-app-capable` for Safari
- Stay with vanilla CSS approach (existing `style.css`), not Tailwind utility classes in RSX
- Auto-refresh via polling (15s interval) rather than WebSocket for delivery list (WebSocket is per-order, not per-courier)

### Risks
- Safari PWA support is limited (no `beforeinstallprompt`) — fallback to manual "Add to Home Screen" instruction
- Service worker caching of WASM bundle needs careful cache versioning (hash in filename helps)
- Dioxus JS interop for `beforeinstallprompt` may need `web-sys` or `wasm-bindgen` calls
- Icon generation without design tools — may use simple SVG placeholder initially

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
