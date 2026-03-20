-- Add metrics columns to orders table for pilot KPI tracking

ALTER TABLE orders ADD COLUMN estimated_eta INTEGER;
ALTER TABLE orders ADD COLUMN actual_delivery_at TEXT;
