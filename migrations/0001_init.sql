-- OpenWok D1 Schema — initial migration
-- Compatible with both rusqlite (local dev) and Cloudflare D1

CREATE TABLE IF NOT EXISTS zones (
    id   TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS restaurants (
    id      TEXT PRIMARY KEY,
    name    TEXT NOT NULL,
    zone_id TEXT NOT NULL REFERENCES zones(id),
    active  INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS menu_items (
    id            TEXT PRIMARY KEY,
    restaurant_id TEXT NOT NULL REFERENCES restaurants(id),
    name          TEXT NOT NULL,
    price         TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS couriers (
    id        TEXT PRIMARY KEY,
    name      TEXT NOT NULL,
    kind      TEXT NOT NULL DEFAULT 'Human',
    zone_id   TEXT NOT NULL REFERENCES zones(id),
    available INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS orders (
    id               TEXT PRIMARY KEY,
    restaurant_id    TEXT NOT NULL REFERENCES restaurants(id),
    courier_id       TEXT REFERENCES couriers(id),
    customer_address TEXT NOT NULL,
    zone_id          TEXT NOT NULL REFERENCES zones(id),
    status           TEXT NOT NULL DEFAULT 'Created',
    food_total       TEXT NOT NULL,
    delivery_fee     TEXT NOT NULL,
    tip              TEXT NOT NULL,
    federal_fee      TEXT NOT NULL,
    local_ops_fee    TEXT NOT NULL,
    processing_fee   TEXT NOT NULL,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS order_items (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id     TEXT NOT NULL REFERENCES orders(id),
    menu_item_id TEXT NOT NULL,
    name         TEXT NOT NULL,
    quantity     INTEGER NOT NULL,
    unit_price   TEXT NOT NULL
);
