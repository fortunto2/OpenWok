# Specification: Offline-First SQLite Cache Layer

**Track ID:** offline-first_20260321
**Type:** Feature
**Created:** 2026-03-21
**Status:** Draft

## Summary

Добавить локальную SQLite базу в браузер/мобилку через `sqlite-wasm-rs` (IndexedDB VFS на WASM, rusqlite на native). Ключевые данные кешируются локально — курьер видит заказы и может нажать "Mark Delivered" даже без сети. Изменения ставятся в очередь и синхронизируются при reconnect.

Не полный CRDT/P2P — простой кеш с outbox-очередью и last-write-wins при конфликтах. Scope ограничен курьерским дашбордом (самый критичный для offline — курьеры теряют сеть в лифтах/подвалах).

## Acceptance Criteria

- [ ] Локальная SQLite база работает в браузере (IndexedDB VFS) через `sqlite-wasm-rs`
- [ ] На native (desktop/iOS) используется `rusqlite` с файловой SQLite — тот же API
- [ ] Миграции: подмножество серверных таблиц (orders, order_items, couriers) создаётся локально
- [ ] Курьерский дашборд (`/my-deliveries`) загружает данные из локальной базы, не ждёт API
- [ ] При online: данные с API сохраняются в локальную базу (pull sync)
- [ ] При offline: "Mark Delivered" сохраняется в outbox-очередь
- [ ] При reconnect: outbox проигрывается через API (push sync)
- [ ] Индикатор online/offline статуса в UI
- [ ] Все 107+ тестов проходят, `make check` чисто
- [ ] WASM build (`dx serve --web`) работает

## Dependencies

- `sqlite-wasm-rs` 0.5 + `sqlite-wasm-vfs` 0.2 (WASM SQLite + IndexedDB)
- `rusqlite` (уже в workspace — для native)
- Существующие API endpoints: `GET /api/my/deliveries`, `GET /api/couriers/me`, `PATCH /api/orders/{id}/status`
- Cross-platform frontend (courier-mobile track — done)

## Out of Scope

- CRDT / conflict resolution (last-write-wins достаточно)
- P2P sync между устройствами
- Offline для customer flow (checkout требует Stripe = online)
- Offline для restaurant owner dashboard
- Service Worker caching (WASM bundle кешируется браузером автоматически)
- Full database replication (кешируем только courier-relevant данные)

## Technical Notes

- `sqlite-wasm-rs` компилируется как `cfg(target_arch = "wasm32")`, `rusqlite` — как `cfg(not(...))`
- IndexedDB VFS (`sqlite-wasm-vfs::RelaxedIdbVfs`) персистит SQLite DB между сессиями
- Outbox pattern: таблица `pending_actions` в локальной базе, проигрывается при online
- Online detection: `navigator.onLine` на WASM, always-online на native (reqwest error = offline fallback)
- Локальная схема — подмножество серверной: только `couriers`, `orders`, `order_items`, `pending_actions`
- Pull sync: при каждом online-запросе обновляем локальную базу
- Push sync: при reconnect перебираем pending_actions и вызываем API
