PRAGMA foreign_keys = ON;

ALTER TABLE products ADD COLUMN reorder_min_millis INTEGER CHECK (reorder_min_millis IS NULL OR reorder_min_millis >= 0);
ALTER TABLE products ADD COLUMN reorder_target_millis INTEGER CHECK (reorder_target_millis IS NULL OR reorder_target_millis >= 0);
ALTER TABLE sales ADD COLUMN payment_method TEXT CHECK (payment_method IN ('CASH', 'TRANSFER', 'CARD', 'OTHER') OR payment_method IS NULL);
ALTER TABLE purchases ADD COLUMN payment_method TEXT CHECK (payment_method IN ('CASH', 'TRANSFER', 'CARD', 'OTHER') OR payment_method IS NULL);

CREATE TABLE expense_categories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(name)) > 0),
  active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

INSERT INTO expense_categories(name) VALUES
  ('Alquiler'), ('Luz'), ('Impuestos'), ('Limpieza'),
  ('Mantenimiento'), ('Sueldos'), ('Otros');

CREATE TABLE expenses (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  occurred_at TEXT NOT NULL,
  category_id INTEGER NOT NULL REFERENCES expense_categories(id) ON DELETE RESTRICT,
  description TEXT NOT NULL CHECK (length(trim(description)) > 0),
  amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
  payment_method TEXT NOT NULL CHECK (payment_method IN ('CASH', 'TRANSFER', 'CARD', 'OTHER')),
  note TEXT,
  status TEXT NOT NULL DEFAULT 'CONFIRMED' CHECK (status = 'CONFIRMED'),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE financial_movements (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  occurred_at TEXT NOT NULL,
  source_type TEXT NOT NULL CHECK (source_type IN ('SALE', 'PURCHASE', 'EXPENSE', 'MANUAL_ADJUSTMENT')),
  source_id INTEGER,
  amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
  direction TEXT NOT NULL CHECK (direction IN ('INCOME', 'OUTFLOW')),
  payment_method TEXT CHECK (payment_method IN ('CASH', 'TRANSFER', 'CARD', 'OTHER') OR payment_method IS NULL),
  description TEXT NOT NULL CHECK (length(trim(description)) > 0),
  note TEXT,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  UNIQUE (source_type, source_id)
);

-- Sprint 1 operations remain honest: they are represented, but their historical
-- payment method stays NULL instead of being guessed as cash.
INSERT INTO financial_movements(occurred_at, source_type, source_id, amount_cents, direction, payment_method, description)
SELECT occurred_at, 'SALE', id, total_cents, 'INCOME', NULL, 'Venta #' || id
FROM sales WHERE status = 'CONFIRMED' AND total_cents > 0;

INSERT INTO financial_movements(occurred_at, source_type, source_id, amount_cents, direction, payment_method, description)
SELECT occurred_at, 'PURCHASE', id, total_cents, 'OUTFLOW', NULL, 'Compra #' || id
FROM purchases WHERE status = 'CONFIRMED' AND total_cents > 0;

CREATE INDEX idx_financial_movements_date ON financial_movements(occurred_at DESC, id DESC);
CREATE INDEX idx_financial_movements_payment_date ON financial_movements(payment_method, occurred_at);
CREATE INDEX idx_expenses_date ON expenses(occurred_at DESC, id DESC);

CREATE TABLE cash_sessions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  business_date TEXT NOT NULL UNIQUE,
  opening_cash_cents INTEGER NOT NULL CHECK (opening_cash_cents >= 0),
  status TEXT NOT NULL DEFAULT 'OPEN' CHECK (status IN ('OPEN', 'CLOSED')),
  expected_cash_cents INTEGER,
  counted_cash_cents INTEGER,
  difference_cents INTEGER,
  opened_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  closed_at TEXT,
  close_note TEXT
);

CREATE TABLE inventory_counts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  occurred_at TEXT NOT NULL,
  note TEXT,
  status TEXT NOT NULL DEFAULT 'CONFIRMED' CHECK (status = 'CONFIRMED'),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE inventory_count_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  inventory_count_id INTEGER NOT NULL REFERENCES inventory_counts(id) ON DELETE RESTRICT,
  product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
  expected_quantity_millis INTEGER NOT NULL,
  counted_quantity_millis INTEGER NOT NULL CHECK (counted_quantity_millis >= 0),
  difference_millis INTEGER NOT NULL,
  unit_cost_cents INTEGER NOT NULL CHECK (unit_cost_cents >= 0),
  difference_value_cents INTEGER NOT NULL,
  UNIQUE (inventory_count_id, product_id)
);

ALTER TABLE inventory_movements ADD COLUMN inventory_count_item_id INTEGER REFERENCES inventory_count_items(id) ON DELETE RESTRICT;
CREATE INDEX idx_inventory_count_items_product ON inventory_count_items(product_id);
CREATE INDEX idx_inventory_counts_date ON inventory_counts(occurred_at DESC, id DESC);

INSERT INTO schema_migrations(version) VALUES (2);
