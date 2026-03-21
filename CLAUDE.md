# CLAUDE.md — OpenWok

Federated food delivery platform with open-book pricing, a $1 federal fee, and local node operators.

## Current Direction

OpenWok currently prioritizes:
- one portable Rust/Dioxus fullstack app
- typed server functions for UI flows
- a separate external API surface without splitting deploy infrastructure yet
- self-hostability outside Cloudflare
- a path toward federation between independently operated nodes

Cloudflare is the current deploy target, not the permanent platform assumption.

## Current Runtime Model

- Main crate: `crates/app`
- External API crate: `crates/api`
- UI: Dioxus 0.7 fullstack
- First load: SSR
- Browser interactivity: WASM hydration
- Server runtime: native Rust binary
- Database: SQLite via `rusqlite`
- Production deploy: Cloudflare Worker front door -> Cloudflare Container -> native `openwok-app`

Important:
- Production is not D1-based right now
- The Worker is only the entrypoint/proxy layer
- The main application logic does not run as a pure Cloudflare Worker runtime
- The current deploy is one process that serves both the fullstack app and `/api/*`

## Cloudflare Conclusions

- Static assets and the web bundle are delivered through Cloudflare edge/CDN
- The current server runtime is centralized in a container, not edge-native compute
- The current container routing uses a singleton container id
- `sleepAfter = "10m"` means the container can sleep when idle and cold-start on the next request
- `max_instances = 3` is configured, but singleton routing means horizontal scaling is effectively not active yet

Important deploy caveat:
- Do not strip the Linux release binary used for the container image
- Stripping breaks Dioxus SSR asset metadata and can cause missing CSS in production
- Deploy commands must keep `CARGO_PROFILE_RELEASE_STRIP=none`

## Federation Position

The current architecture is intentionally more federation-friendly than a pure Worker-only backend because:
- it is easier to self-host on Docker, VMs, bare metal, Fly.io, Railway, etc.
- it avoids hard-locking the main backend model to Cloudflare-specific runtime assumptions
- it keeps node-to-node federation possible over normal server-to-server HTTP/RPC patterns

If the project evolves into a true federation:
- each city/operator can run its own server instance
- protocol and federation APIs should remain platform-neutral
- Cloudflare should stay an optional deployment target, not the only valid runtime

## Repo Structure

```text
crates/
  core/              domain types, pricing, order state machine, repository abstractions
  app/               main Dioxus fullstack app
    src/
      main.rs          server entry + web entry
      app.rs           routes, layout, top-level app state wiring
      state.rs         cart, auth, app mode, platform config
      server_fns/      typed server functions
      pages/           UI surfaces
  api/               external HTTP API + Swagger
    src/
      lib.rs           API router exposed to the app runtime
      main.rs          optional standalone API entrypoint
      db.rs            SQLite migrations and seed data
      sqlite_repo.rs   repository implementation used by both app and API
      payments.rs      payment-aware order creation and Stripe webhook
      state.rs         API state and broadcast wiring
  frontend/          legacy Dioxus SPA
  handlers/          shared axum handlers reused by the API crate
  stripe-universal/  Stripe integration crate

Dockerfile
container-worker.js
wrangler.containers.jsonc
```

## Main Product Routes

- `/` landing page
- `/restaurants` restaurant list
- `/restaurant/:id` menu + cart
- `/checkout` checkout
- `/order/:id` order tracking
- `/economics` public economics page
- `/operator` operator console
- `/login` and `/auth/callback` auth flow
- `/my-restaurants` and `/my-restaurants/:id` owner surfaces
- `/register-courier` and `/my-deliveries` courier surfaces

## Development Commands

```bash
make dev
make dev-api
make test
make clippy
make fmt
make check
make deploy
make setup-hooks
```

Use `make dev` for the main local workflow.

Notes:
- `make dev` runs Tailwind watch and `dx serve` for `crates/app`
- `make dev` is the main path and exposes both UI routes and `/api/*`
- `make dev-api` is only for running the external API by itself when needed
- `make deploy` builds the web bundle, builds the Linux server binary, stages the container image, and deploys with Wrangler

## Quality Gates

- Pre-commit hooks run formatting and clippy
- Keep `cargo fmt --all` clean
- Keep `cargo clippy --workspace -- -D warnings` clean
- Do not leave docs claiming old REST/in-memory/Worker-only architecture as the current main path

## Boundary Rules

- `crates/app` owns UI routes, layout, SSR, hydration, and typed server functions
- `crates/api` owns explicit external HTTP routes, Swagger, webhooks, and integration-facing contracts
- `crates/core` owns business logic and shared types
- `crates/handlers` is shared HTTP handler code used by `crates/api`
- Keep deployment simple unless there is a proven need to split processes
- Do not collapse external API concerns back into ad-hoc UI server functions

## What To Optimize For

When making architecture decisions, optimize for:
- one coherent fullstack app
- one Docker image and one runtime for now
- clear separation between internal UI transport and external API contracts
- typed data flow between UI and server
- portability beyond Cloudflare
- federation-readiness
- edge caching for public/read-heavy traffic

Do not optimize prematurely for:
- one runtime per city
- pure Worker-only execution everywhere
- Cloudflare-specific rewrites unless they materially simplify the current product

## Short Practical Summary

- Use `crates/app` first
- Treat `crates/api` as the current external API crate, not as dead legacy
- Assume Cloudflare Containers is the current production runtime
- Assume static assets are edge-delivered but server compute is container-based
- Assume the main deploy currently serves both app and API from one process
- Assume self-hosting and federation are strategic goals
- Keep docs aligned with this reality
