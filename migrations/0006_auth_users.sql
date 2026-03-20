-- Auth: users table + link orders to users

CREATE TABLE IF NOT EXISTS users (
    id               TEXT PRIMARY KEY,
    supabase_user_id TEXT UNIQUE NOT NULL,
    email            TEXT NOT NULL,
    name             TEXT,
    role             TEXT NOT NULL DEFAULT 'Customer',
    created_at       TEXT NOT NULL
);

ALTER TABLE orders ADD COLUMN user_id TEXT REFERENCES users(id);
