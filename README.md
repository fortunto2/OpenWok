# OpenWok

OpenWok is a food delivery platform focused on transparent pricing, portable infrastructure, and a path toward self-hosted federation.

## Status

- Main runtime: `crates/app` + `crates/api`
- UI: Dioxus 0.7 fullstack with SSR on first load and WASM hydration in the browser
- API: axum routes mounted into the same runtime under `/api/*`
- Database: SQLite via `rusqlite`
- Deploy: Cloudflare Worker entrypoint -> Cloudflare Container -> native Rust server
- Production URL: `https://openwok.superduperai.co`

## What Exists

- Customer flow: `/`, `/restaurants`, `/restaurant/:id`, `/checkout`, `/order/:id`
- Public economics page: `/economics`
- Operator console: `/operator`
- Restaurant owner pages: `/my-restaurants`, `/my-restaurants/:id`, `/onboard-restaurant`
- Courier pages: `/register-courier`, `/my-deliveries`
- Auth pages: `/login`, `/auth/callback`
- External API docs: `/api/docs`

## Repository Layout

```text
crates/
  app/               Dioxus fullstack UI and SSR entrypoint
  api/               external HTTP API mounted into the same runtime
  auth/              shared Supabase auth crate
  core/              domain logic, pricing, order state machine, repo traits
  handlers/          reusable axum handlers used by the API crate
  frontend/          older SPA path, not the primary runtime
  stripe-universal/  Stripe integration crate

container-worker.js        Cloudflare Worker entrypoint
Dockerfile                 runtime image for Cloudflare Containers
wrangler.containers.jsonc  Cloudflare deploy config
planning/ROADMAP.md        current execution roadmap
docs/roadmap-full.md       longer-term product roadmap
```

## Local Development

```bash
# run checks
make check

# main app: UI + SSR + embedded API
make dev

# optional API-only process
make dev-api
```

## Environment

Common local variables:

```bash
APP_BASE_URL=http://127.0.0.1:8080
PUBLIC_APP_URL=http://127.0.0.1:8080
SUPABASE_URL=...
SUPABASE_JWT_ISSUER=...
SUPABASE_ANON_KEY=...
SUPABASE_GOOGLE_AUTH_ENABLED=true
```

Notes:

- `SUPABASE_JWT_SECRET` is optional legacy fallback. Current auth supports JWKS verification.
- Email/password auth works with Supabase email provider.
- Google OAuth can stay enabled in UI and be configured later in Supabase.

## Deploy

```bash
make deploy
```

Current deploy model:

- One Docker image
- One Rust server process
- One SQLite database file inside the container image
- One Cloudflare Worker that routes traffic into the container

Important:

- Do not strip the release server binary used in the container image.
- Dioxus SSR asset metadata breaks when the binary is stripped, which causes missing CSS in production.

## Architectural Notes

- The browser uses WASM.
- The server does not run as WASM; it is a native Linux binary.
- Static assets are still delivered through Cloudflare edge.
- The current container routing is effectively singleton-based and sleeps after inactivity.
- This setup is intentionally more portable than a Cloudflare-only Worker backend.
- The code is already split so UI-specific flows can stay on server functions while external integrations can use `/api/*`.

## Current Direction

- Keep one runtime and one deploy for MVP simplicity.
- Keep the code split into `app`, `api`, `auth`, and `core` so services can be separated later if needed.
- Prefer portable Rust/server architecture now, with room for edge caching and federation later.
