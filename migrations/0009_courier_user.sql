-- Courier-user link + dispatch indexes
-- Links couriers to auth users (like restaurants.owner_id)

ALTER TABLE couriers ADD COLUMN user_id TEXT REFERENCES users(id);

CREATE INDEX IF NOT EXISTS idx_couriers_zone_available ON couriers(zone_id, available);
CREATE INDEX IF NOT EXISTS idx_orders_courier_id ON orders(courier_id);
