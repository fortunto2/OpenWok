# Specification: OpenWok Mobile App (Dioxus Native)

**Track ID:** courier-mobile_20260321
**Type:** Feature
**Created:** 2026-03-21
**Status:** Draft

## Summary

Единое мобильное приложение для iOS (позже Android) на Dioxus 0.7 native. Два режима: "Заказываю" (customer) и "Доставляю" (courier). Вместо PWA-подхода используем `dx build --platform ios` для нативной сборки.

Отдельный крейт `crates/mobile/` — использует `reqwest` (cross-platform) вместо `gloo-net` (web-only), файловое хранение JWT вместо localStorage, и абсолютный URL для API. Переиспользует типы из `openwok-core`.

Текущий `crates/frontend/` остаётся как веб-SPA.

## Acceptance Criteria

- [ ] `crates/mobile/` компилируется и запускается на iOS симуляторе через `dx serve --ios`
- [ ] Google OAuth через системный браузер + deep link callback (`openwok://auth/callback`)
- [ ] Режим "Заказываю": рестораны → меню → корзина → чекаут → трекинг заказа
- [ ] Режим "Доставляю": регистрация курьера, дашборд с toggle availability, mark delivered
- [ ] Переключатель режимов в UI (customer ↔ courier)
- [ ] Все touch targets 48px+, mobile-first CSS с safe area insets
- [ ] Auto-refresh доставок каждые 15 секунд (режим курьера)
- [ ] API через `reqwest` с конфигурируемым base URL — zero web-only deps
- [ ] JWT через файловое хранилище (`directories` crate)
- [ ] `dx bundle --platform ios` собирает .app для TestFlight
- [ ] PWA-артефакты удалены, PRD/roadmap обновлены
- [ ] `make check` проходит (все тесты, clippy, fmt)

## Dependencies

- Courier dispatch backend — complete (courier-dispatch_20260320)
- Auth (Supabase Google OAuth) — complete
- Dioxus 0.7 с мобильной поддержкой
- Xcode + iOS SDK + Rust iOS toolchains (`aarch64-apple-ios`, `aarch64-apple-ios-sim`)
- `openwok-core` — shared domain types

## Out of Scope

- Android (iOS first, Android follow-up)
- Push notifications
- GPS / геолокация
- Offline mode
- Restaurant owner dashboard на мобиле (только web)
- Operator console на мобиле (только web)
- Admin tools на мобиле (только web)
- App Store (TestFlight only for pilot)
- PostHog analytics на мобиле (JS snippet не работает нативно)

## Technical Notes

- Dioxus 0.7 mobile: WebView-based rendering, Rust код ARM64 native
- `dx serve --ios` для dev, `dx bundle --platform ios` для release
- `reqwest` с native TLS — рекомендация Dioxus для cross-platform HTTP
- JWT storage: `directories::ProjectDirs` → `data_dir/jwt.txt`
- OAuth: system browser → Supabase → deep link `openwok://auth/callback?access_token=...`
- API_BASE: абсолютный URL `https://openwok.superduperai.co/api` (не относительный `/api`)
- CORS: backend может потребовать обновления для mobile origin
- Мобильные роуты (9 из 15 веб-роутов): login, callback, restaurants, restaurant/:id, checkout, order/:id, register-courier, my-deliveries, mode-switcher
- Пропускаем: operator, economics, my-restaurants, onboard-restaurant, admin — desktop-only
