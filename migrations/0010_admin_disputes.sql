-- Migration 0010: Admin tools — blocked users + disputes
-- Add blocked column to users table
ALTER TABLE users ADD COLUMN blocked INTEGER NOT NULL DEFAULT 0;

-- Disputes table: linked to orders, created by users, resolved by operators
CREATE TABLE IF NOT EXISTS disputes (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL REFERENCES orders(id),
    user_id TEXT NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Open',
    resolution TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);

CREATE INDEX idx_disputes_order_id ON disputes(order_id);
CREATE INDEX idx_disputes_status ON disputes(status);
