-- Restaurant ownership & extended info
-- Adds owner_id FK to users, plus description, address, phone, timestamps

ALTER TABLE restaurants ADD COLUMN owner_id TEXT REFERENCES users(id);
ALTER TABLE restaurants ADD COLUMN description TEXT;
ALTER TABLE restaurants ADD COLUMN address TEXT;
ALTER TABLE restaurants ADD COLUMN phone TEXT;
ALTER TABLE restaurants ADD COLUMN created_at TEXT;
ALTER TABLE restaurants ADD COLUMN updated_at TEXT;
