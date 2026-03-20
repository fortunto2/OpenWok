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
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            zone_id     TEXT NOT NULL REFERENCES zones(id),
            active      INTEGER NOT NULL DEFAULT 1,
            owner_id    TEXT REFERENCES users(id),
            description TEXT,
            address     TEXT,
            phone       TEXT,
            created_at  TEXT,
            updated_at  TEXT
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
            updated_at       TEXT NOT NULL,
            estimated_eta       INTEGER,
            actual_delivery_at  TEXT
        );

        CREATE TABLE IF NOT EXISTS order_items (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id     TEXT NOT NULL REFERENCES orders(id),
            menu_item_id TEXT NOT NULL,
            name         TEXT NOT NULL,
            quantity     INTEGER NOT NULL,
            unit_price   TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS users (
            id               TEXT PRIMARY KEY,
            supabase_user_id TEXT UNIQUE NOT NULL,
            email            TEXT NOT NULL,
            name             TEXT,
            role             TEXT NOT NULL DEFAULT 'Customer',
            blocked          INTEGER NOT NULL DEFAULT 0,
            created_at       TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS payments (
            id                          TEXT PRIMARY KEY,
            order_id                    TEXT NOT NULL REFERENCES orders(id),
            stripe_payment_intent_id    TEXT,
            stripe_checkout_session_id  TEXT,
            status                      TEXT NOT NULL DEFAULT 'Pending',
            amount_total                TEXT NOT NULL,
            restaurant_amount           TEXT NOT NULL,
            courier_amount              TEXT NOT NULL,
            federal_amount              TEXT NOT NULL,
            local_ops_amount            TEXT NOT NULL,
            processing_amount           TEXT NOT NULL,
            created_at                  TEXT NOT NULL
        );
        ",
    )
    .expect("migration failed");

    // Add user_id column to orders if not present (migration 0006)
    let has_user_id: bool = conn.prepare("SELECT user_id FROM orders LIMIT 0").is_ok();
    if !has_user_id {
        conn.execute_batch("ALTER TABLE orders ADD COLUMN user_id TEXT REFERENCES users(id);")
            .expect("failed to add user_id to orders");
    }

    // Add ownership columns to restaurants if not present (migration 0008)
    let has_owner_id: bool = conn
        .prepare("SELECT owner_id FROM restaurants LIMIT 0")
        .is_ok();
    if !has_owner_id {
        conn.execute_batch(
            "ALTER TABLE restaurants ADD COLUMN owner_id TEXT REFERENCES users(id);
             ALTER TABLE restaurants ADD COLUMN description TEXT;
             ALTER TABLE restaurants ADD COLUMN address TEXT;
             ALTER TABLE restaurants ADD COLUMN phone TEXT;
             ALTER TABLE restaurants ADD COLUMN created_at TEXT;
             ALTER TABLE restaurants ADD COLUMN updated_at TEXT;",
        )
        .expect("failed to add ownership columns to restaurants");
    }

    // Add user_id to couriers + dispatch indexes (migration 0009)
    let has_courier_user_id: bool = conn.prepare("SELECT user_id FROM couriers LIMIT 0").is_ok();
    if !has_courier_user_id {
        conn.execute_batch("ALTER TABLE couriers ADD COLUMN user_id TEXT REFERENCES users(id);")
            .expect("failed to add user_id to couriers");
    }
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_couriers_zone_available ON couriers(zone_id, available);
         CREATE INDEX IF NOT EXISTS idx_orders_courier_id ON orders(courier_id);",
    )
    .expect("failed to create dispatch indexes");

    // Add blocked column to users if not present (migration 0010)
    let has_blocked: bool = conn.prepare("SELECT blocked FROM users LIMIT 0").is_ok();
    if !has_blocked {
        conn.execute_batch("ALTER TABLE users ADD COLUMN blocked INTEGER NOT NULL DEFAULT 0;")
            .expect("failed to add blocked to users");
    }

    // Create disputes table (migration 0010)
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS disputes (
            id          TEXT PRIMARY KEY,
            order_id    TEXT NOT NULL REFERENCES orders(id),
            user_id     TEXT NOT NULL REFERENCES users(id),
            reason      TEXT NOT NULL,
            status      TEXT NOT NULL DEFAULT 'Open',
            resolution  TEXT,
            created_at  TEXT NOT NULL DEFAULT (datetime('now')),
            resolved_at TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_disputes_order_id ON disputes(order_id);
        CREATE INDEX IF NOT EXISTS idx_disputes_status ON disputes(status);",
    )
    .expect("failed to create disputes table");
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

    let zone3_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO zones (id, name) VALUES (?1, ?2)",
        params![zone3_id, "Venice"],
    )
    .unwrap();

    let zone4_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO zones (id, name) VALUES (?1, ?2)",
        params![zone4_id, "Santa Monica"],
    )
    .unwrap();

    let zone5_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO zones (id, name) VALUES (?1, ?2)",
        params![zone5_id, "Koreatown"],
    )
    .unwrap();

    let zone6_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO zones (id, name) VALUES (?1, ?2)",
        params![zone6_id, "Silver Lake"],
    )
    .unwrap();

    type Restaurant<'a> = (&'a str, &'a str, Vec<(&'a str, &'a str)>);
    let restaurants: Vec<Restaurant<'_>> = vec![
        // Downtown LA (3 existing + 2 new)
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
            "Grand Market Noodles",
            &zone_id,
            vec![
                ("Beef Pho", "14.99"),
                ("Dan Dan Noodles", "12.99"),
                ("Wonton Soup", "10.99"),
                ("Char Siu Bao", "7.99"),
                ("Shrimp Dumplings", "11.99"),
            ],
        ),
        (
            "Arts District Smokehouse",
            &zone_id,
            vec![
                ("Brisket Plate", "19.99"),
                ("Pulled Pork Plate", "16.99"),
                ("Smoked Ribs Half Rack", "22.99"),
                ("Cornbread", "4.99"),
                ("Baked Beans", "5.99"),
            ],
        ),
        // Hollywood (1 existing + 1 new)
        (
            "Taco Libre",
            &zone2_id,
            vec![
                ("Street Tacos", "9.99"),
                ("Burrito Bowl", "11.99"),
                ("Churros", "5.99"),
            ],
        ),
        (
            "Sunset Strip Poke",
            &zone2_id,
            vec![
                ("Ahi Poke Bowl", "16.99"),
                ("Salmon Poke Bowl", "15.99"),
                ("Tofu Poke Bowl", "13.99"),
                ("Seaweed Salad", "6.99"),
                ("Coconut Water", "3.99"),
            ],
        ),
        // Venice (3 new)
        (
            "Boardside Burgers",
            &zone3_id,
            vec![
                ("Classic Smash Burger", "11.99"),
                ("Truffle Fries", "7.49"),
                ("BBQ Bacon Burger", "14.99"),
                ("Milkshake", "6.99"),
                ("Chicken Sandwich", "12.49"),
            ],
        ),
        (
            "Venice Pizza Co",
            &zone3_id,
            vec![
                ("Margherita Pizza", "15.99"),
                ("Pepperoni Pizza", "17.99"),
                ("Caesar Salad", "9.99"),
                ("Garlic Knots", "5.99"),
                ("Tiramisu", "8.99"),
            ],
        ),
        (
            "Abbot Kinney Bowls",
            &zone3_id,
            vec![
                ("Acai Bowl", "13.99"),
                ("Poke Bowl", "15.99"),
                ("Smoothie Bowl", "11.99"),
                ("Grain Bowl", "14.49"),
                ("Avocado Toast", "10.99"),
            ],
        ),
        // Santa Monica (3 new)
        (
            "Bay Sushi House",
            &zone4_id,
            vec![
                ("Omakase Roll", "18.99"),
                ("Salmon Sashimi", "16.99"),
                ("Spicy Tuna Roll", "14.99"),
                ("Edamame", "5.99"),
                ("Miso Ramen", "15.99"),
                ("Gyoza", "8.99"),
            ],
        ),
        (
            "Promenade Deli",
            &zone4_id,
            vec![
                ("Turkey Club", "13.99"),
                ("Reuben Sandwich", "14.99"),
                ("Matzo Ball Soup", "9.99"),
                ("Bagel & Lox", "12.99"),
                ("Pastrami on Rye", "15.49"),
            ],
        ),
        (
            "Santa Monica Taqueria",
            &zone4_id,
            vec![
                ("Carne Asada Tacos", "11.99"),
                ("Fish Tacos", "12.99"),
                ("Burrito Supreme", "13.99"),
                ("Elote", "4.99"),
                ("Horchata", "3.99"),
            ],
        ),
        // Koreatown (3 new)
        (
            "Seoul Q BBQ",
            &zone5_id,
            vec![
                ("Galbi Set", "24.99"),
                ("Bulgogi", "19.99"),
                ("Japchae", "13.99"),
                ("Kimchi Jjigae", "14.99"),
                ("Bibimbap", "16.99"),
                ("Korean Fried Tofu", "10.99"),
            ],
        ),
        (
            "K-Bird Fried Chicken",
            &zone5_id,
            vec![
                ("Crispy Chicken Combo", "14.99"),
                ("Spicy Wings", "12.99"),
                ("Chicken Sandwich", "11.99"),
                ("Tteokbokki", "9.99"),
                ("Corn Cheese", "7.99"),
            ],
        ),
        (
            "Bingsu Mountain",
            &zone5_id,
            vec![
                ("Mango Bingsu", "12.99"),
                ("Matcha Bingsu", "13.99"),
                ("Red Bean Bingsu", "11.99"),
                ("Hotteok", "6.99"),
                ("Taro Milk Tea", "5.99"),
            ],
        ),
        // Silver Lake (3 new)
        (
            "Silver Lake Coffee & Bites",
            &zone6_id,
            vec![
                ("Avocado Egg Sandwich", "11.99"),
                ("Oat Milk Latte", "5.99"),
                ("Banana Bread", "4.49"),
                ("Breakfast Burrito", "10.99"),
                ("Granola Parfait", "8.99"),
            ],
        ),
        (
            "Sunset Blvd Thai",
            &zone6_id,
            vec![
                ("Pad See Ew", "13.99"),
                ("Massaman Curry", "15.99"),
                ("Larb Gai", "12.99"),
                ("Mango Sticky Rice", "8.99"),
                ("Thai Iced Tea", "4.99"),
            ],
        ),
        (
            "Hyperion Street Eats",
            &zone6_id,
            vec![
                ("Nashville Hot Chicken", "14.99"),
                ("Mac & Cheese", "8.99"),
                ("Loaded Fries", "9.99"),
                ("Pulled Pork Sandwich", "13.49"),
                ("Coleslaw", "4.99"),
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
        // Tables should exist (6 original + users + payments + disputes = 9)
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('zones','restaurants','menu_items','couriers','orders','order_items','users','payments','disputes')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 9);
    }

    #[test]
    fn seed_data_creates_restaurants() {
        let conn = open(":memory:");
        seed_la_data(&conn);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM restaurants", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 18);
    }

    #[test]
    fn seed_data_idempotent() {
        let conn = open(":memory:");
        seed_la_data(&conn);
        seed_la_data(&conn);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM restaurants", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 18);
    }

    #[test]
    fn seed_creates_menu_items() {
        let conn = open(":memory:");
        seed_la_data(&conn);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM menu_items", [], |r| r.get(0))
            .unwrap();
        assert!(count >= 80); // 18 restaurants with 3-6 items each
    }

    #[test]
    fn seed_creates_zones() {
        let conn = open(":memory:");
        seed_la_data(&conn);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM zones", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 6);
    }
}
