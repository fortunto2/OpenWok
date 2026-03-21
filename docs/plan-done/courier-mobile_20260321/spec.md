# Specification: Cross-Platform Frontend + Config Externalization

**Track ID:** courier-mobile_20260321
**Type:** Refactor
**Created:** 2026-03-21
**Status:** Draft

## Summary

Рефакторинг существующего `crates/frontend/` из web-only в cross-platform Dioxus приложение (web + desktop + iOS). Вместо копирования кода в отдельный крейт — один крейт, разные build targets: `dx serve --web`, `dx serve --desktop`, `dx serve --ios`.

Два направления:
1. **Cross-platform**: заменить web-only зависимости (gloo-net → reqwest, web_sys → cfg-abstractions)
2. **Config externalization**: убрать захардкоженные цены и конфиги (delivery_fee, local_ops_fee, API_BASE, Supabase URL) — фронтенд запрашивает конфиг с бэкенда

## Acceptance Criteria

- [ ] `gloo-net` полностью заменён на `reqwest` — нет web-only HTTP зависимостей
- [ ] Storage (JWT) работает через `cfg`: web_sys на WASM, файловое на native
- [ ] `web_sys::window()` вызовы заменены на platform-agnostic альтернативы
- [ ] Новый endpoint `GET /api/config` возвращает delivery_fee, local_ops_fee, federal_fee
- [ ] Фронтенд получает цены из API, не хардкодит `$5.00` / `$2.50`
- [ ] `API_BASE` конфигурируется (относительный на web, абсолютный на native)
- [ ] `dx serve --web` работает как раньше (регрессий нет)
- [ ] `dx serve --desktop` показывает работающее приложение
- [ ] Мобильная навигация: bottom tab bar + mode switcher (Customer/Courier)
- [ ] Dioxus.toml настроен для multi-platform (bundle identifier, icons)
- [ ] PWA-артефакты удалены, docs обновлены
- [ ] Все 107+ тестов проходят, `make check` чисто

## Dependencies

- Dioxus 0.7 с multi-platform support
- reqwest 0.12 (работает на WASM и native)
- `dirs` crate для native storage
- `open` crate для system browser (OAuth, Stripe)

## Out of Scope

- iOS сборка (нужен Xcode.app — отдельный шаг после установки)
- Android сборка
- Push notifications
- GPS / геолокация
- Offline mode
- PostHog native SDK (analytics будут no-op на native)

## Technical Notes

- `dx serve --web` авто-добавляет feature `web`, target `wasm32-unknown-unknown`
- `dx serve --desktop` авто-добавляет feature `desktop`, target `host`
- `dx serve --ios` авто-добавляет feature `mobile`, target `aarch64-apple-ios-sim`
- reqwest на WASM использует browser fetch API, на native — native TLS. Работает везде
- `uuid` feature `js` нужен только на WASM (crypto.getRandomValues) — сделать conditional
- `web_sys::window().location().set_href()` → `open::that()` (system browser)
- `web_sys::window().location().reload()` → Dioxus navigator или signal refresh
- analytics (PostHog JS) → `cfg(target_arch = "wasm32")` guard, no-op на native
