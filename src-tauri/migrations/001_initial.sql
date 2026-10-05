PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE categories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(name)) > 0),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE products (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL CHECK (length(trim(name)) > 0),
  barcode TEXT COLLATE NOCASE UNIQUE,
  category_id INTEGER REFERENCES categories(id) ON DELETE RESTRICT,
  unit_type TEXT NOT NULL CHECK (unit_type IN ('UNIT', 'WEIGHT')),
  current_cost_cents INTEGER NOT NULL DEFAULT 0 CHECK (current_cost_cents >= 0),
  sale_price_cents INTEGER NOT NULL CHECK (sale_price_cents >= 0),
  active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_products_name ON products(name COLLATE NOCASE);
CREATE INDEX idx_products_category ON products(category_id);

CREATE TABLE purchases (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  occurred_at TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('DRAFT', 'CONFIRMED', 'CANCELLED')),
  total_cents INTEGER NOT NULL DEFAULT 0 CHECK (total_cents >= 0),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  confirmed_at TEXT
);

CREATE TABLE purchase_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  purchase_id INTEGER NOT NULL REFERENCES purchases(id) ON DELETE RESTRICT,
  product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
  quantity_millis INTEGER NOT NULL CHECK (quantity_millis > 0),
  unit_cost_cents INTEGER NOT NULL CHECK (unit_cost_cents >= 0),
  subtotal_cents INTEGER NOT NULL CHECK (subtotal_cents >= 0),
  UNIQUE (purchase_id, product_id)
);

CREATE TABLE sales (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  occurred_at TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('DRAFT', 'CONFIRMED', 'CANCELLED')),
  total_cents INTEGER NOT NULL DEFAULT 0 CHECK (total_cents >= 0),
  total_cost_cents INTEGER NOT NULL DEFAULT 0 CHECK (total_cost_cents >= 0),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  confirmed_at TEXT
);

CREATE TABLE sale_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  sale_id INTEGER NOT NULL REFERENCES sales(id) ON DELETE RESTRICT,
  product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
  product_name TEXT NOT NULL,
  quantity_millis INTEGER NOT NULL CHECK (quantity_millis > 0),
  unit_price_cents INTEGER NOT NULL CHECK (unit_price_cents >= 0),
  unit_cost_cents INTEGER NOT NULL CHECK (unit_cost_cents >= 0),
  subtotal_cents INTEGER NOT NULL CHECK (subtotal_cents >= 0),
  total_cost_cents INTEGER NOT NULL CHECK (total_cost_cents >= 0),
  UNIQUE (sale_id, product_id)
);

CREATE TABLE inventory_movements (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  product_id INTEGER NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
  quantity_millis INTEGER NOT NULL CHECK (quantity_millis <> 0),
  movement_type TEXT NOT NULL CHECK (movement_type IN (
    'INITIAL_STOCK', 'PURCHASE', 'SALE', 'ADJUSTMENT',
    'EXPIRATION', 'BREAKAGE', 'INTERNAL_USE'
  )),
  occurred_at TEXT NOT NULL,
  reference_type TEXT CHECK (reference_type IN ('PURCHASE', 'SALE') OR reference_type IS NULL),
  reference_id INTEGER,
  note TEXT,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  CHECK (
    (reference_type IS NULL AND reference_id IS NULL) OR
    (reference_type IS NOT NULL AND reference_id IS NOT NULL)
  )
);

CREATE INDEX idx_inventory_product_date
  ON inventory_movements(product_id, occurred_at DESC, id DESC);
CREATE INDEX idx_inventory_reference
  ON inventory_movements(reference_type, reference_id);
CREATE INDEX idx_purchase_items_product ON purchase_items(product_id);
CREATE INDEX idx_sale_items_product ON sale_items(product_id);

INSERT INTO schema_migrations(version) VALUES (1);
