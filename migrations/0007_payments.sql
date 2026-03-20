-- Payments: track Stripe payment state per order

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

CREATE INDEX IF NOT EXISTS idx_payments_order_id ON payments(order_id);
CREATE INDEX IF NOT EXISTS idx_payments_stripe_session ON payments(stripe_checkout_session_id);
