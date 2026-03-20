# OpenWok — Full Roadmap

> LA узел → федерация → роботы. $1 federal + local ops. Trusted invites.

## 0) Продукт/правила (1–3 дня)

- [x] "Конституция денег" (6 строк чека): Food / Delivery / Tip / Federal $1 / Local Ops / Processing
- [x] Формула Local Ops Fee для MVP (фикс по зонам)
- [ ] Минимальные SLA метрики: on-time %, ETA error, cancel rate, refund rate, time-to-resolution
- [ ] Базовая политика prohibited items + инцидент-флоу (kill switch, freeze payouts)

## 1) Протокол событий (Event Protocol v0) (3–5 дней)

- [ ] Схемы событий (JSON): OrderCreated / OrderAccepted / CourierAssigned / PickupConfirmed / DropoffConfirmed
- [ ] RefundRequested / RefundResolved
- [ ] PolicyProposed / PolicyActivated
- [ ] InviteCreated / InviteAccepted (для trusted invites)
- [ ] Идентификаторы, идемпотентность, дедупликация
- [ ] Подписи событий (server signing), версионирование схем

## 2) Core backend (LA node) — Rust (2–4 недели)

### Сервисный каркас
- [x] HTTP API (axum) + endpoints
- [ ] OpenAPI спека (utoipa)
- [ ] Auth: users/roles (customer/merchant/courier/operator/admin)
- [ ] Storage: Postgres (sqlx), миграции
- [ ] Очереди/джобы: Redis + background jobs (или NATS)
- [ ] Observability: logs + tracing + metrics

### Домены
- [x] Каталог: рестораны/меню/цены
- [x] Заказы: корзина → заказ → статусы (state machine)
- [x] "Open-book breakdown" вычисление чека (6 строк)
- [ ] Налоги/время готовки в каталоге
- [ ] Диспетч v1: rule-based (ETA, расстояние, fairness, нагрузка)
- [ ] Рефанды/диспуты: флоу + доказательства (фото/гео/таймстемпы)
- [ ] Event log (append-only таблица + projections)

## 3) Клиенты (2–6 недель параллельно)

### Customer web (Dioxus SPA)
- [ ] Каталог ресторанов с фильтром по зоне
- [ ] Меню + корзина + open-book чек
- [ ] Оформление заказа + real-time трекинг (WebSocket)
- [ ] Offline-first: локальный кэш каталога, retry queue

### Courier
- [ ] Принятие заказа, навигация, чек-лист, статусы
- [ ] Инцидент кнопки: "не нашёл", "ресторан задерживает", "клиент не отвечает"
- [ ] Store-and-forward при плохой сети

### Operator console
- [ ] Дашборд SLA, инциденты, возвраты
- [ ] Управление Local Ops Fee (правила/лимиты)
- [ ] Онбординг ресторанов/курьеров (KYB/KYC)
- [ ] Kill switch: отключение мерчанта/курьера, заморозка выплат

## 4) Платежи и выплаты (1–3 недели)

- [ ] Payment flow: merchant of record решение
- [ ] Stripe Connect: split payments (restaurant + courier + federal + local)
- [ ] Процессинг как pass-through строка в чеке
- [ ] Пэйауты курьерам: расписание, статусы, "не удерживаем деньги курьера"
- [ ] Anti-fraud базовый: лимиты, velocity checks, подозрительные паттерны

## 5) Trusted Invites / репутация (1–2 недели)

- [ ] Модель графа приглашений: кто может приглашать кого
- [ ] Правила влияния: лимиты, depth=2, time-decay, штрафы за плохое поведение
- [ ] Репутационные tiers (trust tier отдельно от quality)
- [ ] Апелляции и восстановление (процедуры узла)

## 6) Федерация (узел №2) — после стабильного LA (2–6 недель)

- [ ] Node Operator Agreement (процесс одобрения)
- [ ] Peering allowlist + тех-аудит узла
- [ ] Межузловой каталог (read-only) + переносимость репутации
- [ ] Event relay между узлами (server-to-server, tonic gRPC)
- [ ] CloudEvents формат для межузловых событий
- [ ] ed25519 подписи событий
- [ ] Публичные агрегаты сети (без персональных данных)

## 7) Роботы (волна 2) (пилот 6–12 недель)

- [ ] Тип агента DeliveryAgent: HumanCourier / SidewalkRobot / DroneOperator
- [ ] Зоны робота + исключения (tele-ops интерфейс)
- [ ] Метрики робота: utilization, exception rate, tele-ops cost
- [ ] Партнёрская интеграция (Serve Robotics, Coco/DoorDash)
- [ ] Дроны: только через партнёра Part 135

## 8) LA GTM чеклист

- [ ] Выбрать 1–2 "кармана" района (Downtown LA, Hollywood)
- [ ] 10–20 ресторанов, подписать условия (open-book + fees)
- [ ] Набор первых курьеров (20–40) + обучение
- [ ] Первая неделя: ручной ops — собрать данные
- [ ] Еженедельный отчёт узла: SLA + агрегаты затрат → настройка Local Ops Fee

## Rust стек

| Компонент | Крейт |
|-----------|-------|
| API | axum |
| DB | Postgres + sqlx |
| Cache/Jobs | Redis (позже NATS/JetStream) |
| Auth | JWT + RBAC (scopes) |
| Telemetry | tracing + opentelemetry |
| Event log | append-only таблица + projection tables |
| Federation | tonic (gRPC) + cloudevents-sdk + ed25519-dalek |
| Frontend | Dioxus (SPA) |
