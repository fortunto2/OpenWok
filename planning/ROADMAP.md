# OpenWok Roadmap

This file tracks the current execution roadmap for the product and codebase.

## Principles

- Keep MVP simple: one runtime, one deploy, one database
- Keep boundaries clean: `app`, `api`, `auth`, `core`
- Prefer portable infrastructure over vendor-locked architecture
- Preserve a path to federation and self-hosting
- Add tests at architectural boundaries, not only at the domain layer

## Current State

- Fullstack Dioxus app is the main runtime
- External API is mounted into the same process under `/api/*`
- Cloudflare Containers deployment is working on `https://openwok.superduperai.co`
- Email/password auth works with Supabase
- Google sign-in UI is present and can be enabled fully once Supabase provider setup is finished
- Main order flow, checkout, operator pages, economics page, and production assets are restored

## Now

- Stabilize auth and session flows
- Clean build warnings and minor runtime rough edges
- Add more integration tests around auth, checkout, and order lifecycle
- Keep documentation aligned with actual runtime and deploy model

## Next

### 1. Auth and User Flows

- Add end-to-end happy-path coverage for signup, confirmation, login, and callback
- Add password reset flow
- Add clearer role-aware redirects after sign-in
- Decide which auth flows stay UI-only and which should be exposed via `/api/v1`

### 2. Orders and Operations

- Add integration tests for cart -> checkout -> order -> operator lifecycle
- Move more orchestration into application services in `crates/core`
- Remove remaining duplicated or thin infrastructure wrappers where possible
- Improve operator tooling for order handling and incident visibility

### 3. API Shape

- Keep `/api/*` in the same runtime for now
- Start defining stable versioned routes under `/api/v1`
- Keep Swagger/OpenAPI accurate
- Make external API contracts explicit instead of leaking UI-oriented server function behavior

### 4. Data and Infra

- Decide whether SQLite remains only for MVP or becomes a migration step toward Postgres later
- Make DB, cache, and background job assumptions explicit
- Add a cleaner self-hosted setup story
- Add basic observability for production debugging

### 5. Offline-First and UX

- Improve client-side caching and retry behavior
- Harden checkout and order tracking for flaky network conditions
- Expand PWA/offline behavior only after core flows are stable

### 6. Federation Preparation

- Define the minimum federation protocol and trust model
- Separate internal UI flows from external node-to-node or partner APIs
- Keep portable server architecture so another company can run its own node

## Not Doing Right Now

- Splitting into multiple deploys
- Rebuilding everything as Cloudflare-native Worker-only compute
- Premature city-by-city runtime sharding
- Large infrastructure migrations before MVP flows are stable

## Exit Criteria For The Next Milestone

- Auth flows are predictable and tested
- Checkout and order lifecycle have integration coverage
- `/api/*` contracts are explicit and documented
- Production deploy remains simple and repeatable
- Docs match reality
