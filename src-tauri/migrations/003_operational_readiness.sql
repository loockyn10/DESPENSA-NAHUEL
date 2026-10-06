PRAGMA foreign_keys = ON;
BEGIN IMMEDIATE;

-- Sales and purchases already accept CANCELLED.  Keep that compatible value and
-- add the audit evidence required to explain every cancellation.
ALTER TABLE sales ADD COLUMN voided_at TEXT;
ALTER TABLE sales ADD COLUMN void_reason TEXT;
ALTER TABLE purchases ADD COLUMN voided_at TEXT;
ALTER TABLE purchases ADD COLUMN void_reason TEXT;

-- Only purchases confirmed after this migration have enough evidence for a
-- safe automatic reversal. Historical rows intentionally remain NULL.
ALTER TABLE purchase_items ADD COLUMN previous_stock_millis INTEGER;
ALTER TABLE purchase_items ADD COLUMN previous_cost_cents INTEGER;
ALTER TABLE purchase_items ADD COLUMN resulting_cost_cents INTEGER;
ALTER TABLE purchase_items ADD COLUMN inventory_movement_id INTEGER REFERENCES inventory_movements(id) ON DELETE RESTRICT;

-- Compensation rows are append-only and point at the movement they reverse.
ALTER TABLE inventory_movements ADD COLUMN reversal_of_movement_id INTEGER REFERENCES inventory_movements(id) ON DELETE RESTRICT;
CREATE UNIQUE INDEX idx_inventory_single_reversal
  ON inventory_movements(reversal_of_movement_id)
  WHERE reversal_of_movement_id IS NOT NULL;

ALTER TABLE financial_movements ADD COLUMN reversal_of_movement_id INTEGER REFERENCES financial_movements(id) ON DELETE RESTRICT;
CREATE UNIQUE INDEX idx_financial_single_reversal
  ON financial_movements(reversal_of_movement_id)
  WHERE reversal_of_movement_id IS NOT NULL;

-- Sprint 2 constrained expenses to CONFIRMED. Rebuild only this independent
-- table so it can retain an auditable CANCELLED state.
ALTER TABLE expenses RENAME TO expenses_sprint2;
CREATE TABLE expenses (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  occurred_at TEXT NOT NULL,
  category_id INTEGER NOT NULL REFERENCES expense_categories(id) ON DELETE RESTRICT,
  description TEXT NOT NULL CHECK (length(trim(description)) > 0),
  amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
  payment_method TEXT NOT NULL CHECK (payment_method IN ('CASH', 'TRANSFER', 'CARD', 'OTHER')),
  note TEXT,
  status TEXT NOT NULL DEFAULT 'CONFIRMED' CHECK (status IN ('CONFIRMED', 'CANCELLED')),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  voided_at TEXT,
  void_reason TEXT
);
INSERT INTO expenses(id, occurred_at, category_id, description, amount_cents, payment_method, note, status, created_at)
SELECT id, occurred_at, category_id, description, amount_cents, payment_method, note, status, created_at
FROM expenses_sprint2;
DROP TABLE expenses_sprint2;
CREATE INDEX idx_expenses_date ON expenses(occurred_at DESC, id DESC);

INSERT INTO schema_migrations(version) VALUES (3);
COMMIT;
