use rusqlite::{Connection, params};
use std::path::Path;

pub fn open(path: &str) -> Connection {
    let conn = if path == ":memory:" {
        Connection::open_in_memory().expect("failed to open in-memory db")
    } else {
        let p = Path::new(path);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        Connection::open(p).expect("failed to open database")
    };
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .expect("failed to set pragmas");
    migrate(&conn);
    conn
}

fn migrate(conn: &Connection) {
    conn.execute_batch(
        "
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
        ",
    )
    .expect("migration failed");
}

pub fn seed_la_data(conn: &Connection) {
    // Check if data already seeded
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM restaurants", [], |r| r.get(0))
        .unwrap_or(0);
    if count > 0 {
        return;
    }

    let zone_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO zones (id, name) VALUES (?1, ?2)",
        params![zone_id, "Downtown LA"],
    )
    .unwrap();

    let zone2_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO zones (id, name) VALUES (?1, ?2)",
        params![zone2_id, "Hollywood"],
    )
    .unwrap();

    let restaurants = [
        (
            "Pad Thai Palace",
            &zone_id,
            vec![
                ("Pad Thai", "12.99"),
                ("Tom Yum Soup", "8.99"),
                ("Green Curry", "14.99"),
            ],
        ),
        (
            "Sushi Wave",
            &zone_id,
            vec![
                ("California Roll", "10.99"),
                ("Salmon Nigiri", "13.99"),
                ("Miso Soup", "4.99"),
            ],
        ),
        (
            "Taco Libre",
            &zone2_id,
            vec![
                ("Street Tacos", "9.99"),
                ("Burrito Bowl", "11.99"),
                ("Churros", "5.99"),
            ],
        ),
    ];

    for (name, zid, items) in &restaurants {
        let rid = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO restaurants (id, name, zone_id, active) VALUES (?1, ?2, ?3, 1)",
            params![rid, name, zid],
        )
        .unwrap();

        for (item_name, price) in items {
            let mid = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO menu_items (id, restaurant_id, name, price) VALUES (?1, ?2, ?3, ?4)",
                params![mid, rid, item_name, price],
            )
            .unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_run_without_error() {
        let conn = open(":memory:");
        // Tables should exist
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('zones','restaurants','menu_items','couriers','orders','order_items')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 6);
    }

    #[test]
    fn seed_data_creates_restaurants() {
        let conn = open(":memory:");
        seed_la_data(&conn);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM restaurants", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn seed_data_idempotent() {
        let conn = open(":memory:");
        seed_la_data(&conn);
        seed_la_data(&conn);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM restaurants", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn seed_creates_menu_items() {
        let conn = open(":memory:");
        seed_la_data(&conn);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM menu_items", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 9); // 3 restaurants * 3 items
    }

    #[test]
    fn seed_creates_zones() {
        let conn = open(":memory:");
        seed_la_data(&conn);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM zones", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }
}
