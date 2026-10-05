use crate::models::*;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

const MIGRATION_001: &str = include_str!("../migrations/001_initial.sql");

pub type DbResult<T> = Result<T, String>;

fn db_error(error: rusqlite::Error) -> String {
    let message = error.to_string();
    if message.contains("products.barcode") {
        "Ese código de barras ya está asignado a otro producto.".to_string()
    } else if message.contains("categories.name") {
        "Ya existe una categoría con ese nombre.".to_string()
    } else {
        format!("Error de base de datos: {message}")
    }
}

pub fn open(path: &Path) -> DbResult<Connection> {
    let connection = Connection::open(path).map_err(db_error)?;
    configure(&connection)?;
    Ok(connection)
}

fn configure(connection: &Connection) -> DbResult<()> {
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(db_error)?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(db_error)?;
    Ok(())
}

pub fn migrate(connection: &Connection) -> DbResult<()> {
    let has_migrations: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations')",
            [],
            |row| row.get(0),
        )
        .map_err(db_error)?;
    if !has_migrations {
        connection.execute_batch(MIGRATION_001).map_err(db_error)?;
    }
    Ok(())
}

fn required_text(value: &str, label: &str) -> DbResult<String> {
    let value = value.trim();
    if value.is_empty() {
        Err(format!("{label} es obligatorio."))
    } else {
        Ok(value.to_string())
    }
}

fn optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn validate_unit_type(unit_type: &str) -> DbResult<()> {
    match unit_type {
        "UNIT" | "WEIGHT" => Ok(()),
        _ => Err("El tipo debe ser UNIT o WEIGHT.".to_string()),
    }
}

fn validate_quantity(unit_type: &str, quantity_millis: i64) -> DbResult<()> {
    if quantity_millis <= 0 {
        return Err("La cantidad debe ser mayor que cero.".to_string());
    }
    if unit_type == "UNIT" && quantity_millis % 1000 != 0 {
        return Err("Los productos por unidad no admiten cantidades fraccionarias.".to_string());
    }
    Ok(())
}

fn checked_amount(quantity_millis: i64, unit_cents: i64) -> DbResult<i64> {
    if quantity_millis < 0 || unit_cents < 0 {
        return Err("Cantidad e importe no pueden ser negativos.".to_string());
    }
    let numerator = i128::from(quantity_millis) * i128::from(unit_cents);
    let amount = (numerator + 500) / 1000;
    i64::try_from(amount).map_err(|_| "El importe calculado es demasiado grande.".to_string())
}

pub fn weighted_average_cost(
    current_stock_millis: i64,
    current_cost_cents: i64,
    purchased_millis: i64,
    purchase_cost_cents: i64,
) -> DbResult<i64> {
    if purchased_millis <= 0 || purchase_cost_cents < 0 {
        return Err("La compra debe tener cantidad positiva y costo no negativo.".to_string());
    }
    if current_stock_millis <= 0 || current_cost_cents == 0 {
        return Ok(purchase_cost_cents);
    }
    let total_quantity = i128::from(current_stock_millis) + i128::from(purchased_millis);
    let weighted = i128::from(current_stock_millis) * i128::from(current_cost_cents)
        + i128::from(purchased_millis) * i128::from(purchase_cost_cents);
    let rounded = (weighted + total_quantity / 2) / total_quantity;
    i64::try_from(rounded)
        .map_err(|_| "El costo promedio calculado es demasiado grande.".to_string())
}

fn stock_for_product(connection: &Connection, product_id: i64) -> DbResult<i64> {
    connection
        .query_row(
            "SELECT COALESCE(SUM(quantity_millis), 0) FROM inventory_movements WHERE product_id = ?1",
            [product_id],
            |row| row.get(0),
        )
        .map_err(db_error)
}

fn product_unit(connection: &Connection, product_id: i64) -> DbResult<String> {
    connection
        .query_row(
            "SELECT unit_type FROM products WHERE id = ?1",
            [product_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(db_error)?
        .ok_or_else(|| "El producto no existe.".to_string())
}

pub fn list_categories(connection: &Connection) -> DbResult<Vec<Category>> {
    let mut statement = connection
        .prepare("SELECT id, name FROM categories ORDER BY name COLLATE NOCASE")
        .map_err(db_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;
    Ok(rows)
}

pub fn create_category(connection: &Connection, name: &str) -> DbResult<Category> {
    let name = required_text(name, "El nombre")?;
    connection
        .execute("INSERT INTO categories(name) VALUES (?1)", [&name])
        .map_err(db_error)?;
    Ok(Category {
        id: connection.last_insert_rowid(),
        name,
    })
}

pub fn update_category(connection: &Connection, id: i64, name: &str) -> DbResult<Category> {
    let name = required_text(name, "El nombre")?;
    let changed = connection
        .execute(
            "UPDATE categories SET name = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?2",
            params![name, id],
        )
        .map_err(db_error)?;
    if changed == 0 {
        return Err("La categoría no existe.".to_string());
    }
    Ok(Category { id, name })
}

const PRODUCT_SELECT: &str = "
    SELECT p.id, p.name, p.barcode, p.category_id, c.name, p.unit_type,
           p.current_cost_cents, p.sale_price_cents, p.active,
           COALESCE(SUM(m.quantity_millis), 0), p.created_at, p.updated_at
    FROM products p
    LEFT JOIN categories c ON c.id = p.category_id
    LEFT JOIN inventory_movements m ON m.product_id = p.id
    GROUP BY p.id
    ORDER BY p.name COLLATE NOCASE";

pub fn list_products(connection: &Connection) -> DbResult<Vec<Product>> {
    let mut statement = connection.prepare(PRODUCT_SELECT).map_err(db_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(Product {
                id: row.get(0)?,
                name: row.get(1)?,
                barcode: row.get(2)?,
                category_id: row.get(3)?,
                category_name: row.get(4)?,
                unit_type: row.get(5)?,
                current_cost_cents: row.get(6)?,
                sale_price_cents: row.get(7)?,
                active: row.get::<_, i64>(8)? == 1,
                stock_millis: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;
    Ok(rows)
}

pub fn create_product(connection: &Connection, input: ProductInput) -> DbResult<i64> {
    let name = required_text(&input.name, "El nombre")?;
    validate_unit_type(&input.unit_type)?;
    if input.sale_price_cents < 0 {
        return Err("El precio no puede ser negativo.".to_string());
    }
    let barcode = optional_text(input.barcode);
    connection
        .execute(
            "INSERT INTO products(name, barcode, category_id, unit_type, sale_price_cents) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, barcode, input.category_id, input.unit_type, input.sale_price_cents],
        )
        .map_err(db_error)?;
    Ok(connection.last_insert_rowid())
}

pub fn update_product(connection: &Connection, id: i64, input: ProductInput) -> DbResult<()> {
    let name = required_text(&input.name, "El nombre")?;
    validate_unit_type(&input.unit_type)?;
    if input.sale_price_cents < 0 {
        return Err("El precio no puede ser negativo.".to_string());
    }
    let barcode = optional_text(input.barcode);
    let changed = connection
        .execute(
            "UPDATE products SET name = ?1, barcode = ?2, category_id = ?3, unit_type = ?4,
             sale_price_cents = ?5, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?6",
            params![name, barcode, input.category_id, input.unit_type, input.sale_price_cents, id],
        )
        .map_err(db_error)?;
    if changed == 0 {
        return Err("El producto no existe.".to_string());
    }
    Ok(())
}

pub fn set_product_active(connection: &Connection, id: i64, active: bool) -> DbResult<()> {
    let changed = connection
        .execute(
            "UPDATE products SET active = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?2",
            params![active, id],
        )
        .map_err(db_error)?;
    if changed == 0 {
        return Err("El producto no existe.".to_string());
    }
    Ok(())
}

pub fn add_initial_stock(connection: &mut Connection, input: StockInput) -> DbResult<()> {
    let unit_type = product_unit(connection, input.product_id)?;
    validate_quantity(&unit_type, input.quantity_millis)?;
    let occurred_at = required_text(&input.occurred_at, "La fecha")?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    let movement_count: i64 = transaction
        .query_row(
            "SELECT COUNT(*) FROM inventory_movements WHERE product_id = ?1",
            [input.product_id],
            |row| row.get(0),
        )
        .map_err(db_error)?;
    if movement_count > 0 {
        return Err(
            "El stock inicial sólo puede cargarse antes del primer movimiento. Usá un ajuste."
                .to_string(),
        );
    }
    transaction
        .execute(
            "INSERT INTO inventory_movements(product_id, quantity_millis, movement_type, occurred_at, note)
             VALUES (?1, ?2, 'INITIAL_STOCK', ?3, ?4)",
            params![input.product_id, input.quantity_millis, occurred_at, optional_text(input.note)],
        )
        .map_err(db_error)?;
    transaction.commit().map_err(db_error)
}

pub fn adjust_stock(connection: &mut Connection, input: StockInput) -> DbResult<()> {
    let unit_type = product_unit(connection, input.product_id)?;
    if input.quantity_millis < 0 {
        return Err("El stock real no puede ser negativo.".to_string());
    }
    if unit_type == "UNIT" && input.quantity_millis % 1000 != 0 {
        return Err("Los productos por unidad no admiten cantidades fraccionarias.".to_string());
    }
    let occurred_at = required_text(&input.occurred_at, "La fecha")?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    let current = stock_for_product(&transaction, input.product_id)?;
    let difference = input.quantity_millis - current;
    if difference == 0 {
        return Err(
            "El stock real coincide con el registrado; no hay ajuste para guardar.".to_string(),
        );
    }
    transaction
        .execute(
            "INSERT INTO inventory_movements(product_id, quantity_millis, movement_type, occurred_at, note)
             VALUES (?1, ?2, 'ADJUSTMENT', ?3, ?4)",
            params![input.product_id, difference, occurred_at, optional_text(input.note)],
        )
        .map_err(db_error)?;
    transaction.commit().map_err(db_error)
}

pub fn list_movements(
    connection: &Connection,
    product_id: Option<i64>,
) -> DbResult<Vec<InventoryMovement>> {
    let sql = "SELECT m.id, m.product_id, p.name, m.quantity_millis, m.movement_type,
                      m.occurred_at, m.reference_type, m.reference_id, m.note
               FROM inventory_movements m JOIN products p ON p.id = m.product_id
               WHERE (?1 IS NULL OR m.product_id = ?1)
               ORDER BY m.occurred_at DESC, m.id DESC LIMIT 100";
    let mut statement = connection.prepare(sql).map_err(db_error)?;
    let rows = statement
        .query_map([product_id], |row| {
            Ok(InventoryMovement {
                id: row.get(0)?,
                product_id: row.get(1)?,
                product_name: row.get(2)?,
                quantity_millis: row.get(3)?,
                movement_type: row.get(4)?,
                occurred_at: row.get(5)?,
                reference_type: row.get(6)?,
                reference_id: row.get(7)?,
                note: row.get(8)?,
            })
        })
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;
    Ok(rows)
}

fn validate_unique_items(items: &[LineInput]) -> DbResult<()> {
    if items.is_empty() {
        return Err("Agregá al menos un producto.".to_string());
    }
    let mut products = HashSet::new();
    if items.iter().any(|item| !products.insert(item.product_id)) {
        return Err("Cada producto debe aparecer una sola vez.".to_string());
    }
    Ok(())
}

pub fn confirm_purchase(
    connection: &mut Connection,
    input: OperationInput,
) -> DbResult<OperationSummary> {
    validate_unique_items(&input.items)?;
    let occurred_at = required_text(&input.occurred_at, "La fecha")?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    let mut prepared = Vec::new();
    let mut total_cents = 0_i64;
    for item in &input.items {
        let (unit_type, current_cost, active): (String, i64, bool) = transaction
            .query_row(
                "SELECT unit_type, current_cost_cents, active FROM products WHERE id = ?1",
                [item.product_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "Uno de los productos no existe.".to_string())?;
        if !active {
            return Err("No se puede comprar un producto inactivo.".to_string());
        }
        validate_quantity(&unit_type, item.quantity_millis)?;
        let unit_cost = item
            .unit_cost_cents
            .ok_or_else(|| "Ingresá el costo unitario.".to_string())?;
        let subtotal = checked_amount(item.quantity_millis, unit_cost)?;
        total_cents = total_cents
            .checked_add(subtotal)
            .ok_or_else(|| "El total es demasiado grande.".to_string())?;
        let current_stock = stock_for_product(&transaction, item.product_id)?;
        let new_cost =
            weighted_average_cost(current_stock, current_cost, item.quantity_millis, unit_cost)?;
        prepared.push((
            item.product_id,
            item.quantity_millis,
            unit_cost,
            subtotal,
            new_cost,
        ));
    }
    transaction
        .execute(
            "INSERT INTO purchases(occurred_at, status, total_cents, confirmed_at)
             VALUES (?1, 'CONFIRMED', ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            params![occurred_at, total_cents],
        )
        .map_err(db_error)?;
    let purchase_id = transaction.last_insert_rowid();
    for (product_id, quantity, unit_cost, subtotal, new_cost) in prepared {
        transaction
            .execute(
                "INSERT INTO purchase_items(purchase_id, product_id, quantity_millis, unit_cost_cents, subtotal_cents)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![purchase_id, product_id, quantity, unit_cost, subtotal],
            )
            .map_err(db_error)?;
        transaction
            .execute(
                "INSERT INTO inventory_movements(product_id, quantity_millis, movement_type, occurred_at, reference_type, reference_id)
                 VALUES (?1, ?2, 'PURCHASE', ?3, 'PURCHASE', ?4)",
                params![product_id, quantity, occurred_at, purchase_id],
            )
            .map_err(db_error)?;
        transaction
            .execute(
                "UPDATE products SET current_cost_cents = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?2",
                params![new_cost, product_id],
            )
            .map_err(db_error)?;
    }
    transaction.commit().map_err(db_error)?;
    Ok(OperationSummary {
        id: purchase_id,
        occurred_at,
        status: "CONFIRMED".to_string(),
        total_cents,
        total_cost_cents: None,
        item_count: input.items.len() as i64,
    })
}

pub fn list_purchases(connection: &Connection) -> DbResult<Vec<OperationSummary>> {
    let mut statement = connection
        .prepare(
            "SELECT p.id, p.occurred_at, p.status, p.total_cents, COUNT(i.id)
             FROM purchases p LEFT JOIN purchase_items i ON i.purchase_id = p.id
             GROUP BY p.id ORDER BY p.occurred_at DESC, p.id DESC LIMIT 50",
        )
        .map_err(db_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(OperationSummary {
                id: row.get(0)?,
                occurred_at: row.get(1)?,
                status: row.get(2)?,
                total_cents: row.get(3)?,
                total_cost_cents: None,
                item_count: row.get(4)?,
            })
        })
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;
    Ok(rows)
}

pub fn confirm_sale(
    connection: &mut Connection,
    input: OperationInput,
) -> DbResult<OperationSummary> {
    validate_unique_items(&input.items)?;
    let occurred_at = required_text(&input.occurred_at, "La fecha")?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    let mut prepared = Vec::new();
    let mut total_cents = 0_i64;
    let mut total_cost_cents = 0_i64;
    for item in &input.items {
        let (name, unit_type, price, cost, active): (String, String, i64, i64, bool) = transaction
            .query_row(
                "SELECT name, unit_type, sale_price_cents, current_cost_cents, active FROM products WHERE id = ?1",
                [item.product_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "Uno de los productos no existe.".to_string())?;
        if !active {
            return Err(format!("{name} está inactivo."));
        }
        validate_quantity(&unit_type, item.quantity_millis)?;
        let stock = stock_for_product(&transaction, item.product_id)?;
        if stock < item.quantity_millis {
            return Err(format!("Stock insuficiente para {name}."));
        }
        let subtotal = checked_amount(item.quantity_millis, price)?;
        let total_cost = checked_amount(item.quantity_millis, cost)?;
        total_cents = total_cents
            .checked_add(subtotal)
            .ok_or_else(|| "El total es demasiado grande.".to_string())?;
        total_cost_cents = total_cost_cents
            .checked_add(total_cost)
            .ok_or_else(|| "El costo total es demasiado grande.".to_string())?;
        prepared.push((
            item.product_id,
            name,
            item.quantity_millis,
            price,
            cost,
            subtotal,
            total_cost,
        ));
    }
    transaction
        .execute(
            "INSERT INTO sales(occurred_at, status, total_cents, total_cost_cents, confirmed_at)
             VALUES (?1, 'CONFIRMED', ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            params![occurred_at, total_cents, total_cost_cents],
        )
        .map_err(db_error)?;
    let sale_id = transaction.last_insert_rowid();
    for (product_id, name, quantity, price, cost, subtotal, total_cost) in prepared {
        transaction
            .execute(
                "INSERT INTO sale_items(sale_id, product_id, product_name, quantity_millis, unit_price_cents,
                 unit_cost_cents, subtotal_cents, total_cost_cents) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![sale_id, product_id, name, quantity, price, cost, subtotal, total_cost],
            )
            .map_err(db_error)?;
        transaction
            .execute(
                "INSERT INTO inventory_movements(product_id, quantity_millis, movement_type, occurred_at, reference_type, reference_id)
                 VALUES (?1, ?2, 'SALE', ?3, 'SALE', ?4)",
                params![product_id, -quantity, occurred_at, sale_id],
            )
            .map_err(db_error)?;
    }
    transaction.commit().map_err(db_error)?;
    Ok(OperationSummary {
        id: sale_id,
        occurred_at,
        status: "CONFIRMED".to_string(),
        total_cents,
        total_cost_cents: Some(total_cost_cents),
        item_count: input.items.len() as i64,
    })
}

pub fn list_sales(connection: &Connection) -> DbResult<Vec<OperationSummary>> {
    let mut statement = connection
        .prepare(
            "SELECT s.id, s.occurred_at, s.status, s.total_cents, s.total_cost_cents, COUNT(i.id)
             FROM sales s LEFT JOIN sale_items i ON i.sale_id = s.id
             GROUP BY s.id ORDER BY s.occurred_at DESC, s.id DESC LIMIT 50",
        )
        .map_err(db_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(OperationSummary {
                id: row.get(0)?,
                occurred_at: row.get(1)?,
                status: row.get(2)?,
                total_cents: row.get(3)?,
                total_cost_cents: row.get(4)?,
                item_count: row.get(5)?,
            })
        })
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;
    Ok(rows)
}

pub fn get_sale(connection: &Connection, id: i64) -> DbResult<SaleDetail> {
    let (occurred_at, total_cents, total_cost_cents): (String, i64, i64) = connection
        .query_row(
            "SELECT occurred_at, total_cents, total_cost_cents FROM sales WHERE id = ?1 AND status = 'CONFIRMED'",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(db_error)?
        .ok_or_else(|| "La venta no existe.".to_string())?;
    let mut statement = connection
        .prepare(
            "SELECT product_id, product_name, quantity_millis, unit_price_cents, unit_cost_cents,
                    subtotal_cents, total_cost_cents FROM sale_items WHERE sale_id = ?1 ORDER BY id",
        )
        .map_err(db_error)?;
    let items = statement
        .query_map([id], |row| {
            Ok(SaleItemSnapshot {
                product_id: row.get(0)?,
                product_name: row.get(1)?,
                quantity_millis: row.get(2)?,
                unit_price_cents: row.get(3)?,
                unit_cost_cents: row.get(4)?,
                subtotal_cents: row.get(5)?,
                total_cost_cents: row.get(6)?,
            })
        })
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;
    Ok(SaleDetail {
        id,
        occurred_at,
        total_cents,
        total_cost_cents,
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn database() -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        configure(&connection).unwrap();
        migrate(&connection).unwrap();
        connection
    }

    fn product(connection: &Connection, name: &str, barcode: Option<&str>) -> i64 {
        create_product(
            connection,
            ProductInput {
                name: name.to_string(),
                barcode: barcode.map(str::to_string),
                category_id: None,
                unit_type: "UNIT".to_string(),
                sale_price_cents: 160_000,
            },
        )
        .unwrap()
    }

    #[test]
    fn weighted_average_matches_example() {
        assert_eq!(
            weighted_average_cost(10_000, 100_000, 10_000, 120_000).unwrap(),
            110_000
        );
        assert_eq!(
            weighted_average_cost(0, 0, 10_000, 120_000).unwrap(),
            120_000
        );
        assert_eq!(
            weighted_average_cost(10_000, 0, 10_000, 120_000).unwrap(),
            120_000
        );
    }

    #[test]
    fn purchase_adds_stock_cost_and_movement() {
        let mut connection = database();
        let id = product(&connection, "Coca-Cola 2.25 L", Some("7790001"));
        add_initial_stock(
            &mut connection,
            StockInput {
                product_id: id,
                quantity_millis: 10_000,
                occurred_at: "2026-10-05".into(),
                note: None,
            },
        )
        .unwrap();
        connection
            .execute(
                "UPDATE products SET current_cost_cents = 100000 WHERE id = ?1",
                [id],
            )
            .unwrap();
        confirm_purchase(
            &mut connection,
            OperationInput {
                occurred_at: "2026-10-05".into(),
                items: vec![LineInput {
                    product_id: id,
                    quantity_millis: 10_000,
                    unit_cost_cents: Some(120_000),
                }],
            },
        )
        .unwrap();
        assert_eq!(stock_for_product(&connection, id).unwrap(), 20_000);
        let cost: i64 = connection
            .query_row(
                "SELECT current_cost_cents FROM products WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cost, 110_000);
        let count: i64 = connection.query_row("SELECT COUNT(*) FROM inventory_movements WHERE product_id = ?1 AND movement_type = 'PURCHASE'", [id], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn sale_reduces_stock_and_keeps_cost_snapshot() {
        let mut connection = database();
        let id = product(&connection, "Coca-Cola 2.25 L", None);
        add_initial_stock(
            &mut connection,
            StockInput {
                product_id: id,
                quantity_millis: 10_000,
                occurred_at: "2026-10-05".into(),
                note: None,
            },
        )
        .unwrap();
        connection
            .execute(
                "UPDATE products SET current_cost_cents = 100000 WHERE id = ?1",
                [id],
            )
            .unwrap();
        let sale = confirm_sale(
            &mut connection,
            OperationInput {
                occurred_at: "2026-10-05".into(),
                items: vec![LineInput {
                    product_id: id,
                    quantity_millis: 2_000,
                    unit_cost_cents: None,
                }],
            },
        )
        .unwrap();
        connection
            .execute(
                "UPDATE products SET current_cost_cents = 120000 WHERE id = ?1",
                [id],
            )
            .unwrap();
        assert_eq!(stock_for_product(&connection, id).unwrap(), 8_000);
        assert_eq!(
            get_sale(&connection, sale.id).unwrap().items[0].unit_cost_cents,
            100_000
        );
        let count: i64 = connection.query_row("SELECT COUNT(*) FROM inventory_movements WHERE reference_type = 'SALE' AND reference_id = ?1", [sale.id], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn failed_sale_is_atomic() {
        let mut connection = database();
        let first = product(&connection, "Con stock", None);
        let second = product(&connection, "Sin stock", None);
        add_initial_stock(
            &mut connection,
            StockInput {
                product_id: first,
                quantity_millis: 1_000,
                occurred_at: "2026-10-05".into(),
                note: None,
            },
        )
        .unwrap();
        let result = confirm_sale(
            &mut connection,
            OperationInput {
                occurred_at: "2026-10-05".into(),
                items: vec![
                    LineInput {
                        product_id: first,
                        quantity_millis: 1_000,
                        unit_cost_cents: None,
                    },
                    LineInput {
                        product_id: second,
                        quantity_millis: 1_000,
                        unit_cost_cents: None,
                    },
                ],
            },
        );
        assert!(result.is_err());
        assert_eq!(stock_for_product(&connection, first).unwrap(), 1_000);
        let sales: i64 = connection
            .query_row("SELECT COUNT(*) FROM sales", [], |r| r.get(0))
            .unwrap();
        assert_eq!(sales, 0);
    }

    #[test]
    fn barcode_is_unique() {
        let connection = database();
        product(&connection, "Uno", Some("ABC"));
        let duplicate = create_product(
            &connection,
            ProductInput {
                name: "Dos".into(),
                barcode: Some("abc".into()),
                category_id: None,
                unit_type: "UNIT".into(),
                sale_price_cents: 1,
            },
        );
        assert!(duplicate.is_err());
    }

    #[test]
    fn end_to_end_data_persists_after_reopen() {
        let unique = format!(
            "despensa-nahuel-test-{}-{}.sqlite3",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        let sale_id;
        {
            let mut connection = open(&path).unwrap();
            migrate(&connection).unwrap();
            let category = create_category(&connection, "Bebidas").unwrap();
            let product_id = create_product(
                &connection,
                ProductInput {
                    name: "Coca-Cola 2.25 L".into(),
                    barcode: Some("7790000002250".into()),
                    category_id: Some(category.id),
                    unit_type: "UNIT".into(),
                    sale_price_cents: 160_000,
                },
            )
            .unwrap();
            add_initial_stock(
                &mut connection,
                StockInput {
                    product_id,
                    quantity_millis: 10_000,
                    occurred_at: "2026-10-05".into(),
                    note: Some("Conteo inicial".into()),
                },
            )
            .unwrap();
            confirm_purchase(
                &mut connection,
                OperationInput {
                    occurred_at: "2026-10-05".into(),
                    items: vec![LineInput {
                        product_id,
                        quantity_millis: 10_000,
                        unit_cost_cents: Some(120_000),
                    }],
                },
            )
            .unwrap();
            sale_id = confirm_sale(
                &mut connection,
                OperationInput {
                    occurred_at: "2026-10-05".into(),
                    items: vec![LineInput {
                        product_id,
                        quantity_millis: 2_000,
                        unit_cost_cents: None,
                    }],
                },
            )
            .unwrap()
            .id;
        }
        {
            let connection = open(&path).unwrap();
            migrate(&connection).unwrap();
            let products = list_products(&connection).unwrap();
            assert_eq!(products.len(), 1);
            assert_eq!(products[0].stock_millis, 18_000);
            assert_eq!(products[0].current_cost_cents, 120_000);
            let sale = get_sale(&connection, sale_id).unwrap();
            assert_eq!(sale.items[0].unit_cost_cents, 120_000);
            let movements = list_movements(&connection, Some(products[0].id)).unwrap();
            assert!(movements
                .iter()
                .any(|movement| movement.movement_type == "SALE"));
        }
        std::fs::remove_file(path).unwrap();
    }
}
