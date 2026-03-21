# Specification: Dioxus Fullstack + Cloudflare Containers Migration

**Track ID:** fullstack-migration_20260321
**Type:** Refactor
**Created:** 2026-03-21
**Status:** Draft

## Summary

Миграция с текущей архитектуры (SPA + отдельный API Worker) на Dioxus Fullstack в Docker контейнере на Cloudflare Containers. Один бинарь = SSR + server functions + WASM гидратация. Убирает REST API boilerplate, cached_get велосипед, sync.rs, local_db.rs. Каждая нода федерации = один Container instance.

Текущая архитектура: 5 крейтов (core, handlers, api, frontend, worker) + 2 деплоя (Worker + frontend assets). Целевая: 2 крейта (core, app) + 1 деплой (Docker container).

## Acceptance Criteria

- [ ] Единый крейт `crates/app` с features `server` и `web`
- [ ] `#[server_fn]` вместо REST API — минимум для: restaurants, orders, couriers, auth, config
- [ ] SSR: `/restaurants` отдаёт HTML с данными (SEO-friendly)
- [ ] WASM гидратация на клиенте (интерактивность)
- [ ] `dx serve` в dev: SSR + hot reload работает
- [ ] Dockerfile для `linux/amd64` — билд fullstack бинаря
- [ ] `wrangler.jsonc` — конфигурация Cloudflare Container
- [ ] SQLite persistence (rusqlite) — те же миграции
- [ ] Stripe payments работают через server functions
- [ ] WebSocket для order tracking (или server-sent events)
- [ ] `make check` проходит (core тесты, clippy, fmt)
- [ ] Старые крейты (api, handlers, frontend, worker) можно удалить

## Dependencies

- Dioxus 0.7 с `fullstack` feature
- axum 0.8 (server side)
- rusqlite (SQLite, уже используется в api crate)
- Docker (для сборки контейнера)
- wrangler CLI (для деплоя контейнера)
- Cloudflare Containers beta access

## Out of Scope

- Federation protocol (node-to-node sync) — отдельный трек
- Mobile native build (iOS/Android) — сохраняем возможность через `web` feature
- PostHog native analytics
- Push notifications

## Technical Notes

### Dioxus Fullstack структура
```toml
[features]
server = ["dioxus/server", "dep:axum", "dep:tokio", "dep:rusqlite"]
web = ["dioxus/web"]
```

### Server Functions заменяют REST API
```rust
// Вместо: GET /api/restaurants → reqwest → JSON → parse
// Теперь:
#[server]
async fn get_restaurants() -> ServerFnResult<Vec<Restaurant>> {
    let db = extract::<SqlitePool>().await?;
    Ok(db.list_restaurants().await?)
}
// В компоненте:
let restaurants = use_server_future(get_restaurants)?;
```

### Что переиспользуется
- `crates/core` — все типы, pricing, order state machine (без изменений)
- `crates/api/src/sqlite_repo.rs` — Repository impl (переносится в app)
- `crates/api/src/db.rs` — миграции (переносится)
- `crates/frontend/src/pages/*` — RSX компоненты (адаптируются)
- `migrations/*.sql` — без изменений

### Что удаляется
- `crates/frontend/src/api.rs` — заменяется server functions
- `crates/frontend/src/local_db.rs` — SSR делает ненужным
- `crates/frontend/src/sync.rs` — SSR делает ненужным
- `crates/frontend/src/platform.rs` — SSR делает ненужным
- `crates/handlers/` — заменяется server functions
- `crates/worker/` — заменяется Container
- `crates/api/` — сливается в app

### Docker + Cloudflare Containers
```dockerfile
FROM rust:slim AS builder
RUN cargo build --release --features server
FROM debian:bookworm-slim
COPY --from=builder /target/release/openwok /app
CMD ["/app"]
```

```jsonc
// wrangler.jsonc
{
  "containers": [{
    "class_name": "OpenWokNode",
    "image": "./Dockerfile",
    "max_instances": 3
  }]
}
```
