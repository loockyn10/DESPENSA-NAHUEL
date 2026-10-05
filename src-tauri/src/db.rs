use crate::models::*;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

const MIGRATION_001: &str = include_str!("../migrations/001_initial.sql");
const MIGRATION_002: &str = include_str!("../migrations/002_business_control.sql");

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
    let has_v2: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 2)",
            [],
            |row| row.get(0),
        )
        .map_err(db_error)?;
    if !has_v2 {
        connection.execute_batch(MIGRATION_002).map_err(db_error)?;
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

fn validate_payment_method(payment_method: &str) -> DbResult<()> {
    match payment_method {
        "CASH" | "TRANSFER" | "CARD" | "OTHER" => Ok(()),
        _ => Err("Elegí un medio de pago válido.".to_string()),
    }
}

fn validate_reorder(minimum: Option<i64>, target: Option<i64>, unit_type: &str) -> DbResult<()> {
    if minimum.is_some_and(|value| value < 0) || target.is_some_and(|value| value < 0) {
        return Err("Los niveles de reposición no pueden ser negativos.".to_string());
    }
    if unit_type == "UNIT"
        && (minimum.is_some_and(|value| value % 1000 != 0)
            || target.is_some_and(|value| value % 1000 != 0))
    {
        return Err("Los productos por unidad no admiten cantidades fraccionarias.".to_string());
    }
    if let (Some(minimum), Some(target)) = (minimum, target) {
        if target < minimum {
            return Err("El stock objetivo debe ser mayor o igual al stock mínimo.".to_string());
        }
    }
    Ok(())
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
           COALESCE(SUM(m.quantity_millis), 0), p.reorder_min_millis,
           p.reorder_target_millis, p.created_at, p.updated_at
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
                reorder_min_millis: row.get(10)?,
                reorder_target_millis: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
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
    validate_reorder(
        input.reorder_min_millis,
        input.reorder_target_millis,
        &input.unit_type,
    )?;
    if input.sale_price_cents < 0 {
        return Err("El precio no puede ser negativo.".to_string());
    }
    let barcode = optional_text(input.barcode);
    connection
        .execute(
            "INSERT INTO products(name, barcode, category_id, unit_type, sale_price_cents, reorder_min_millis, reorder_target_millis)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![name, barcode, input.category_id, input.unit_type, input.sale_price_cents, input.reorder_min_millis, input.reorder_target_millis],
        )
        .map_err(db_error)?;
    Ok(connection.last_insert_rowid())
}

pub fn update_product(connection: &Connection, id: i64, input: ProductInput) -> DbResult<()> {
    let name = required_text(&input.name, "El nombre")?;
    validate_unit_type(&input.unit_type)?;
    validate_reorder(
        input.reorder_min_millis,
        input.reorder_target_millis,
        &input.unit_type,
    )?;
    if input.sale_price_cents < 0 {
        return Err("El precio no puede ser negativo.".to_string());
    }
    let barcode = optional_text(input.barcode);
    let changed = connection
        .execute(
            "UPDATE products SET name = ?1, barcode = ?2, category_id = ?3, unit_type = ?4,
             sale_price_cents = ?5, reorder_min_millis = ?6, reorder_target_millis = ?7,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?8",
            params![
                name,
                barcode,
                input.category_id,
                input.unit_type,
                input.sale_price_cents,
                input.reorder_min_millis,
                input.reorder_target_millis,
                id
            ],
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
    validate_payment_method(&input.payment_method)?;
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
            "INSERT INTO purchases(occurred_at, status, total_cents, confirmed_at, payment_method)
             VALUES (?1, 'CONFIRMED', ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), ?3)",
            params![occurred_at, total_cents, input.payment_method],
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
    transaction
        .execute(
            "INSERT INTO financial_movements(occurred_at, source_type, source_id, amount_cents, direction, payment_method, description)
             VALUES (?1, 'PURCHASE', ?2, ?3, 'OUTFLOW', ?4, ?5)",
            params![occurred_at, purchase_id, total_cents, input.payment_method, format!("Compra #{purchase_id}")],
        )
        .map_err(db_error)?;
    transaction.commit().map_err(db_error)?;
    Ok(OperationSummary {
        id: purchase_id,
        occurred_at,
        status: "CONFIRMED".to_string(),
        total_cents,
        total_cost_cents: None,
        item_count: input.items.len() as i64,
        payment_method: Some(input.payment_method),
    })
}

pub fn list_purchases(connection: &Connection) -> DbResult<Vec<OperationSummary>> {
    let mut statement = connection
        .prepare(
            "SELECT p.id, p.occurred_at, p.status, p.total_cents, COUNT(i.id), p.payment_method
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
                payment_method: row.get(5)?,
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
    validate_payment_method(&input.payment_method)?;
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
            "INSERT INTO sales(occurred_at, status, total_cents, total_cost_cents, confirmed_at, payment_method)
             VALUES (?1, 'CONFIRMED', ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), ?4)",
            params![occurred_at, total_cents, total_cost_cents, input.payment_method],
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
    transaction
        .execute(
            "INSERT INTO financial_movements(occurred_at, source_type, source_id, amount_cents, direction, payment_method, description)
             VALUES (?1, 'SALE', ?2, ?3, 'INCOME', ?4, ?5)",
            params![occurred_at, sale_id, total_cents, input.payment_method, format!("Venta #{sale_id}")],
        )
        .map_err(db_error)?;
    transaction.commit().map_err(db_error)?;
    Ok(OperationSummary {
        id: sale_id,
        occurred_at,
        status: "CONFIRMED".to_string(),
        total_cents,
        total_cost_cents: Some(total_cost_cents),
        item_count: input.items.len() as i64,
        payment_method: Some(input.payment_method),
    })
}

pub fn list_sales(connection: &Connection) -> DbResult<Vec<OperationSummary>> {
    let mut statement = connection
        .prepare(
            "SELECT s.id, s.occurred_at, s.status, s.total_cents, s.total_cost_cents, COUNT(i.id), s.payment_method
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
                payment_method: row.get(6)?,
            })
        })
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;
    Ok(rows)
}

pub fn get_sale(connection: &Connection, id: i64) -> DbResult<SaleDetail> {
    let (occurred_at, total_cents, total_cost_cents, payment_method): (String, i64, i64, Option<String>) = connection
        .query_row(
            "SELECT occurred_at, total_cents, total_cost_cents, payment_method FROM sales WHERE id = ?1 AND status = 'CONFIRMED'",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
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
        payment_method,
        items,
    })
}

pub fn list_expense_categories(connection: &Connection) -> DbResult<Vec<ExpenseCategory>> {
    let mut statement = connection
        .prepare(
            "SELECT id, name FROM expense_categories WHERE active = 1 ORDER BY name COLLATE NOCASE",
        )
        .map_err(db_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(ExpenseCategory {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;
    Ok(rows)
}

pub fn create_expense_category(connection: &Connection, name: &str) -> DbResult<ExpenseCategory> {
    let name = required_text(name, "El nombre")?;
    connection
        .execute("INSERT INTO expense_categories(name) VALUES (?1)", [&name])
        .map_err(db_error)?;
    Ok(ExpenseCategory {
        id: connection.last_insert_rowid(),
        name,
    })
}

pub fn create_expense(connection: &mut Connection, input: ExpenseInput) -> DbResult<Expense> {
    let occurred_at = required_text(&input.occurred_at, "La fecha")?;
    let description = required_text(&input.description, "La descripción")?;
    validate_payment_method(&input.payment_method)?;
    if input.amount_cents <= 0 {
        return Err("El importe debe ser mayor que cero.".to_string());
    }
    let note = optional_text(input.note);
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    let category_name: String = transaction
        .query_row(
            "SELECT name FROM expense_categories WHERE id = ?1 AND active = 1",
            [input.category_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(db_error)?
        .ok_or_else(|| "La categoría de gasto no existe.".to_string())?;
    transaction
        .execute(
            "INSERT INTO expenses(occurred_at, category_id, description, amount_cents, payment_method, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![occurred_at, input.category_id, description, input.amount_cents, input.payment_method, note],
        )
        .map_err(db_error)?;
    let id = transaction.last_insert_rowid();
    transaction
        .execute(
            "INSERT INTO financial_movements(occurred_at, source_type, source_id, amount_cents, direction, payment_method, description, note)
             VALUES (?1, 'EXPENSE', ?2, ?3, 'OUTFLOW', ?4, ?5, ?6)",
            params![occurred_at, id, input.amount_cents, input.payment_method, format!("Gasto — {description}"), note],
        )
        .map_err(db_error)?;
    transaction.commit().map_err(db_error)?;
    Ok(Expense {
        id,
        occurred_at,
        category_id: input.category_id,
        category_name,
        description,
        amount_cents: input.amount_cents,
        payment_method: input.payment_method,
        note,
    })
}

pub fn list_expenses(connection: &Connection) -> DbResult<Vec<Expense>> {
    let mut statement = connection
        .prepare(
            "SELECT e.id, e.occurred_at, e.category_id, c.name, e.description,
                    e.amount_cents, e.payment_method, e.note
             FROM expenses e JOIN expense_categories c ON c.id = e.category_id
             WHERE e.status = 'CONFIRMED' ORDER BY e.occurred_at DESC, e.id DESC LIMIT 100",
        )
        .map_err(db_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(Expense {
                id: row.get(0)?,
                occurred_at: row.get(1)?,
                category_id: row.get(2)?,
                category_name: row.get(3)?,
                description: row.get(4)?,
                amount_cents: row.get(5)?,
                payment_method: row.get(6)?,
                note: row.get(7)?,
            })
        })
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;
    Ok(rows)
}

pub fn list_financial_movements(connection: &Connection) -> DbResult<Vec<FinancialMovement>> {
    let mut statement = connection
        .prepare(
            "SELECT id, occurred_at, source_type, source_id, amount_cents, direction,
                    payment_method, description, note
             FROM financial_movements ORDER BY occurred_at DESC, id DESC LIMIT 150",
        )
        .map_err(db_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(FinancialMovement {
                id: row.get(0)?,
                occurred_at: row.get(1)?,
                source_type: row.get(2)?,
                source_id: row.get(3)?,
                amount_cents: row.get(4)?,
                direction: row.get(5)?,
                payment_method: row.get(6)?,
                description: row.get(7)?,
                note: row.get(8)?,
            })
        })
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;
    Ok(rows)
}

fn cash_summary(connection: &Connection, business_date: &str) -> DbResult<CashSummary> {
    let session: Option<(i64, String, i64, Option<i64>, Option<i64>, Option<i64>)> = connection
        .query_row(
            "SELECT id, status, opening_cash_cents, expected_cash_cents, counted_cash_cents, difference_cents
             FROM cash_sessions WHERE business_date = ?1",
            [business_date],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .optional()
        .map_err(db_error)?;
    let (income, outflow): (i64, i64) = connection
        .query_row(
            "SELECT
               COALESCE(SUM(CASE WHEN direction = 'INCOME' THEN amount_cents ELSE 0 END), 0),
               COALESCE(SUM(CASE WHEN direction = 'OUTFLOW' THEN amount_cents ELSE 0 END), 0)
             FROM financial_movements WHERE payment_method = 'CASH' AND substr(occurred_at, 1, 10) = ?1",
            [business_date],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(db_error)?;
    let (session_id, status, opening, stored_expected, counted, difference) = match session {
        Some((id, status, opening, stored_expected, counted, difference)) => (
            Some(id),
            Some(status),
            opening,
            stored_expected,
            counted,
            difference,
        ),
        None => (None, None, 0, None, None, None),
    };
    let expected = if status.as_deref() == Some("CLOSED") {
        stored_expected.unwrap_or(opening + income - outflow)
    } else {
        opening + income - outflow
    };
    Ok(CashSummary {
        session_id,
        business_date: business_date.to_string(),
        status,
        opening_cash_cents: opening,
        cash_income_cents: income,
        cash_outflow_cents: outflow,
        expected_cash_cents: expected,
        counted_cash_cents: counted,
        difference_cents: difference,
    })
}

pub fn get_cash_summary(connection: &Connection, business_date: &str) -> DbResult<CashSummary> {
    cash_summary(connection, &required_text(business_date, "La fecha")?)
}

pub fn open_cash_session(
    connection: &Connection,
    business_date: &str,
    opening_cash_cents: i64,
) -> DbResult<CashSummary> {
    let business_date = required_text(business_date, "La fecha")?;
    if opening_cash_cents < 0 {
        return Err("El saldo inicial no puede ser negativo.".to_string());
    }
    connection
        .execute(
            "INSERT INTO cash_sessions(business_date, opening_cash_cents) VALUES (?1, ?2)",
            params![business_date, opening_cash_cents],
        )
        .map_err(db_error)?;
    cash_summary(connection, &business_date)
}

pub fn add_manual_cash_movement(
    connection: &Connection,
    input: ManualCashInput,
) -> DbResult<FinancialMovement> {
    let occurred_at = required_text(&input.occurred_at, "La fecha")?;
    let note = required_text(&input.note, "El motivo")?;
    if input.amount_cents <= 0 {
        return Err("El importe debe ser mayor que cero.".to_string());
    }
    if input.direction != "INCOME" && input.direction != "OUTFLOW" {
        return Err("El ajuste debe ser un ingreso o un egreso.".to_string());
    }
    let open: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM cash_sessions WHERE business_date = substr(?1, 1, 10) AND status = 'OPEN')",
            [&occurred_at], |row| row.get(0),
        )
        .map_err(db_error)?;
    if !open {
        return Err("Abrí la caja del día antes de registrar un ajuste.".to_string());
    }
    let description = if input.direction == "INCOME" {
        "Ingreso manual de caja"
    } else {
        "Egreso manual de caja"
    };
    connection
        .execute(
            "INSERT INTO financial_movements(occurred_at, source_type, amount_cents, direction, payment_method, description, note)
             VALUES (?1, 'MANUAL_ADJUSTMENT', ?2, ?3, 'CASH', ?4, ?5)",
            params![occurred_at, input.amount_cents, input.direction, description, note],
        )
        .map_err(db_error)?;
    Ok(FinancialMovement {
        id: connection.last_insert_rowid(),
        occurred_at,
        source_type: "MANUAL_ADJUSTMENT".into(),
        source_id: None,
        amount_cents: input.amount_cents,
        direction: input.direction,
        payment_method: Some("CASH".into()),
        description: description.into(),
        note: Some(note),
    })
}

pub fn close_cash_session(
    connection: &mut Connection,
    business_date: &str,
    counted_cash_cents: i64,
    note: Option<String>,
) -> DbResult<CashSummary> {
    let business_date = required_text(business_date, "La fecha")?;
    if counted_cash_cents < 0 {
        return Err("El efectivo contado no puede ser negativo.".to_string());
    }
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    let summary = cash_summary(&transaction, &business_date)?;
    let session_id = summary
        .session_id
        .ok_or_else(|| "La caja del día no está abierta.".to_string())?;
    if summary.status.as_deref() != Some("OPEN") {
        return Err("La caja del día ya está cerrada.".to_string());
    }
    let difference = counted_cash_cents - summary.expected_cash_cents;
    transaction
        .execute(
            "UPDATE cash_sessions SET status = 'CLOSED', expected_cash_cents = ?1,
             counted_cash_cents = ?2, difference_cents = ?3, close_note = ?4,
             closed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?5 AND status = 'OPEN'",
            params![
                summary.expected_cash_cents,
                counted_cash_cents,
                difference,
                optional_text(note),
                session_id
            ],
        )
        .map_err(db_error)?;
    transaction.commit().map_err(db_error)?;
    cash_summary(connection, &business_date)
}

fn signed_value(quantity_millis: i64, unit_cost_cents: i64) -> DbResult<i64> {
    let absolute = checked_amount(quantity_millis.abs(), unit_cost_cents)?;
    Ok(if quantity_millis < 0 {
        -absolute
    } else {
        absolute
    })
}

pub fn confirm_inventory_count(
    connection: &mut Connection,
    input: InventoryCountInput,
) -> DbResult<InventoryCountSummary> {
    if input.items.is_empty() {
        return Err("Agregá al menos un producto contado.".to_string());
    }
    let occurred_at = required_text(&input.occurred_at, "La fecha")?;
    let mut unique = HashSet::new();
    if input
        .items
        .iter()
        .any(|line| !unique.insert(line.product_id))
    {
        return Err("Cada producto debe aparecer una sola vez.".to_string());
    }
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    transaction
        .execute(
            "INSERT INTO inventory_counts(occurred_at, note) VALUES (?1, ?2)",
            params![occurred_at, optional_text(input.note)],
        )
        .map_err(db_error)?;
    let count_id = transaction.last_insert_rowid();
    let mut negative_value_cents = 0;
    let mut positive_value_cents = 0;
    for line in &input.items {
        let (unit_type, cost): (String, i64) = transaction
            .query_row(
                "SELECT unit_type, current_cost_cents FROM products WHERE id = ?1",
                [line.product_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| "Uno de los productos no existe.".to_string())?;
        if line.counted_quantity_millis < 0
            || (unit_type == "UNIT" && line.counted_quantity_millis % 1000 != 0)
        {
            return Err("La cantidad contada no es válida para el producto.".to_string());
        }
        let expected = stock_for_product(&transaction, line.product_id)?;
        let difference = line.counted_quantity_millis - expected;
        let value = signed_value(difference, cost)?;
        transaction.execute(
            "INSERT INTO inventory_count_items(inventory_count_id, product_id, expected_quantity_millis,
             counted_quantity_millis, difference_millis, unit_cost_cents, difference_value_cents)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![count_id, line.product_id, expected, line.counted_quantity_millis, difference, cost, value],
        ).map_err(db_error)?;
        let item_id = transaction.last_insert_rowid();
        if difference != 0 {
            transaction.execute(
                "INSERT INTO inventory_movements(product_id, quantity_millis, movement_type, occurred_at, note, inventory_count_item_id)
                 VALUES (?1, ?2, 'ADJUSTMENT', ?3, ?4, ?5)",
                params![line.product_id, difference, occurred_at, format!("Control de inventario #{count_id}"), item_id],
            ).map_err(db_error)?;
        }
        if value < 0 {
            negative_value_cents += -value;
        } else {
            positive_value_cents += value;
        }
    }
    transaction.commit().map_err(db_error)?;
    Ok(InventoryCountSummary {
        id: count_id,
        occurred_at,
        item_count: input.items.len() as i64,
        negative_value_cents,
        positive_value_cents,
    })
}

pub fn list_inventory_counts(connection: &Connection) -> DbResult<Vec<InventoryCountSummary>> {
    let mut statement = connection.prepare(
        "SELECT c.id, c.occurred_at, COUNT(i.id),
          COALESCE(SUM(CASE WHEN i.difference_value_cents < 0 THEN -i.difference_value_cents ELSE 0 END), 0),
          COALESCE(SUM(CASE WHEN i.difference_value_cents > 0 THEN i.difference_value_cents ELSE 0 END), 0)
         FROM inventory_counts c LEFT JOIN inventory_count_items i ON i.inventory_count_id = c.id
         GROUP BY c.id ORDER BY c.occurred_at DESC, c.id DESC LIMIT 50"
    ).map_err(db_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(InventoryCountSummary {
                id: row.get(0)?,
                occurred_at: row.get(1)?,
                item_count: row.get(2)?,
                negative_value_cents: row.get(3)?,
                positive_value_cents: row.get(4)?,
            })
        })
        .map_err(db_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db_error)?;
    Ok(rows)
}

pub fn list_replenishment(connection: &Connection) -> DbResult<Vec<ReplenishmentItem>> {
    let mut items = Vec::new();
    for product in list_products(connection)? {
        if !product.active {
            continue;
        }
        let (Some(minimum), Some(target)) =
            (product.reorder_min_millis, product.reorder_target_millis)
        else {
            continue;
        };
        if product.stock_millis <= minimum {
            let suggested = (target - product.stock_millis).max(0);
            items.push(ReplenishmentItem {
                product_id: product.id,
                product_name: product.name,
                unit_type: product.unit_type,
                stock_millis: product.stock_millis,
                reorder_min_millis: minimum,
                reorder_target_millis: target,
                suggested_quantity_millis: suggested,
                current_cost_cents: product.current_cost_cents,
                estimated_cost_cents: checked_amount(suggested, product.current_cost_cents)?,
            });
        }
    }
    items.sort_by(|a, b| {
        b.estimated_cost_cents
            .cmp(&a.estimated_cost_cents)
            .then_with(|| a.product_name.cmp(&b.product_name))
    });
    Ok(items)
}

pub fn dashboard_summary(
    connection: &Connection,
    start_date: &str,
    end_date_exclusive: &str,
) -> DbResult<DashboardSummary> {
    let start_date = required_text(start_date, "La fecha inicial")?;
    let end_date_exclusive = required_text(end_date_exclusive, "La fecha final")?;
    if start_date >= end_date_exclusive {
        return Err("El rango de fechas no es válido.".to_string());
    }
    let (sales, cost): (i64, i64) = connection
        .query_row(
            "SELECT COALESCE(SUM(total_cents), 0), COALESCE(SUM(total_cost_cents), 0)
         FROM sales WHERE status = 'CONFIRMED' AND occurred_at >= ?1 AND occurred_at < ?2",
            params![start_date, end_date_exclusive],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(db_error)?;
    let expenses: i64 = connection.query_row(
        "SELECT COALESCE(SUM(amount_cents), 0) FROM expenses WHERE status = 'CONFIRMED' AND occurred_at >= ?1 AND occurred_at < ?2",
        params![start_date, end_date_exclusive], |row| row.get(0),
    ).map_err(db_error)?;
    let purchases: i64 = connection.query_row(
        "SELECT COALESCE(SUM(total_cents), 0) FROM purchases WHERE status = 'CONFIRMED' AND occurred_at >= ?1 AND occurred_at < ?2",
        params![start_date, end_date_exclusive], |row| row.get(0),
    ).map_err(db_error)?;
    let (cash_in, cash_out, unknown): (i64, i64, i64) = connection
        .query_row(
            "SELECT
          COALESCE(SUM(CASE WHEN direction = 'INCOME' THEN amount_cents ELSE 0 END), 0),
          COALESCE(SUM(CASE WHEN direction = 'OUTFLOW' THEN amount_cents ELSE 0 END), 0),
          COALESCE(SUM(CASE WHEN payment_method IS NULL THEN 1 ELSE 0 END), 0)
         FROM financial_movements WHERE occurred_at >= ?1 AND occurred_at < ?2",
            params![start_date, end_date_exclusive],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(db_error)?;
    let (negative, positive): (i64, i64) = connection.query_row(
        "SELECT
          COALESCE(SUM(CASE WHEN i.difference_value_cents < 0 THEN -i.difference_value_cents ELSE 0 END), 0),
          COALESCE(SUM(CASE WHEN i.difference_value_cents > 0 THEN i.difference_value_cents ELSE 0 END), 0)
         FROM inventory_count_items i JOIN inventory_counts c ON c.id = i.inventory_count_id
         WHERE c.occurred_at >= ?1 AND c.occurred_at < ?2",
        params![start_date, end_date_exclusive], |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(db_error)?;
    let inventory_value =
        list_products(connection)?
            .into_iter()
            .try_fold(0_i64, |total, product| {
                total
                    .checked_add(checked_amount(
                        product.stock_millis.max(0),
                        product.current_cost_cents,
                    )?)
                    .ok_or_else(|| "El valor de inventario es demasiado grande.".to_string())
            })?;
    let replenishment_count = list_replenishment(connection)?.len() as i64;
    Ok(DashboardSummary {
        start_date,
        end_date_exclusive,
        sales_cents: sales,
        cost_of_goods_cents: cost,
        gross_profit_cents: sales - cost,
        expenses_cents: expenses,
        estimated_result_cents: sales - cost - expenses,
        purchases_cents: purchases,
        cash_in_cents: cash_in,
        cash_out_cents: cash_out,
        inventory_value_cents: inventory_value,
        replenishment_count,
        negative_inventory_difference_cents: negative,
        positive_inventory_difference_cents: positive,
        unknown_payment_count: unknown,
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
                reorder_min_millis: None,
                reorder_target_millis: None,
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
                payment_method: "TRANSFER".into(),
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
                payment_method: "CASH".into(),
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
                payment_method: "CASH".into(),
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
                reorder_min_millis: None,
                reorder_target_millis: None,
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
                    reorder_min_millis: Some(5_000),
                    reorder_target_millis: Some(20_000),
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
                    payment_method: "TRANSFER".into(),
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
                    payment_method: "CASH".into(),
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

    #[test]
    fn economic_result_uses_sale_snapshot_and_expenses_not_purchases() {
        let mut connection = database();
        let id = product(&connection, "Producto", None);
        connection.execute("UPDATE products SET sale_price_cents = 150000, current_cost_cents = 100000 WHERE id = ?1", [id]).unwrap();
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
        confirm_purchase(
            &mut connection,
            OperationInput {
                occurred_at: "2026-10-05".into(),
                payment_method: "TRANSFER".into(),
                items: vec![LineInput {
                    product_id: id,
                    quantity_millis: 5_000,
                    unit_cost_cents: Some(100_000),
                }],
            },
        )
        .unwrap();
        confirm_sale(
            &mut connection,
            OperationInput {
                occurred_at: "2026-10-05".into(),
                payment_method: "CASH".into(),
                items: vec![LineInput {
                    product_id: id,
                    quantity_millis: 1_000,
                    unit_cost_cents: None,
                }],
            },
        )
        .unwrap();
        let category = list_expense_categories(&connection).unwrap()[0].id;
        create_expense(
            &mut connection,
            ExpenseInput {
                occurred_at: "2026-10-05".into(),
                category_id: category,
                description: "Servicio".into(),
                amount_cents: 20_000,
                payment_method: "CASH".into(),
                note: None,
            },
        )
        .unwrap();
        let dashboard = dashboard_summary(&connection, "2026-10-01", "2026-11-01").unwrap();
        assert_eq!(dashboard.sales_cents, 150_000);
        assert_eq!(dashboard.cost_of_goods_cents, 100_000);
        assert_eq!(dashboard.expenses_cents, 20_000);
        assert_eq!(dashboard.estimated_result_cents, 30_000);
        assert_eq!(dashboard.purchases_cents, 500_000);
    }

    #[test]
    fn cash_only_counts_cash_and_close_keeps_difference() {
        let mut connection = database();
        let id = product(&connection, "Producto", None);
        connection
            .execute(
                "UPDATE products SET sale_price_cents = 100000 WHERE id = ?1",
                [id],
            )
            .unwrap();
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
        open_cash_session(&connection, "2026-10-05", 2_000_000).unwrap();
        confirm_sale(
            &mut connection,
            OperationInput {
                occurred_at: "2026-10-05".into(),
                payment_method: "CASH".into(),
                items: vec![LineInput {
                    product_id: id,
                    quantity_millis: 10_000,
                    unit_cost_cents: None,
                }],
            },
        )
        .unwrap();
        let second = product(&connection, "Transferencia", None);
        add_initial_stock(
            &mut connection,
            StockInput {
                product_id: second,
                quantity_millis: 1_000,
                occurred_at: "2026-10-05".into(),
                note: None,
            },
        )
        .unwrap();
        confirm_sale(
            &mut connection,
            OperationInput {
                occurred_at: "2026-10-05".into(),
                payment_method: "TRANSFER".into(),
                items: vec![LineInput {
                    product_id: second,
                    quantity_millis: 1_000,
                    unit_cost_cents: None,
                }],
            },
        )
        .unwrap();
        let category = list_expense_categories(&connection).unwrap()[0].id;
        create_expense(
            &mut connection,
            ExpenseInput {
                occurred_at: "2026-10-05".into(),
                category_id: category,
                description: "Gasto".into(),
                amount_cents: 200_000,
                payment_method: "CASH".into(),
                note: None,
            },
        )
        .unwrap();
        let open = get_cash_summary(&connection, "2026-10-05").unwrap();
        assert_eq!(open.expected_cash_cents, 2_800_000);
        let closed = close_cash_session(
            &mut connection,
            "2026-10-05",
            2_750_000,
            Some("Cierre".into()),
        )
        .unwrap();
        assert_eq!(closed.difference_cents, Some(-50_000));
    }

    #[test]
    fn physical_count_creates_adjustment_and_replenishment() {
        let mut connection = database();
        let id = create_product(
            &connection,
            ProductInput {
                name: "Coca-Cola".into(),
                barcode: None,
                category_id: None,
                unit_type: "UNIT".into(),
                sale_price_cents: 100,
                reorder_min_millis: Some(5_000),
                reorder_target_millis: Some(20_000),
            },
        )
        .unwrap();
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
        confirm_inventory_count(
            &mut connection,
            InventoryCountInput {
                occurred_at: "2026-10-05".into(),
                note: None,
                items: vec![InventoryCountLineInput {
                    product_id: id,
                    counted_quantity_millis: 3_000,
                }],
            },
        )
        .unwrap();
        assert_eq!(stock_for_product(&connection, id).unwrap(), 3_000);
        let movement: i64 = connection.query_row(
            "SELECT quantity_millis FROM inventory_movements WHERE inventory_count_item_id IS NOT NULL", [], |row| row.get(0)
        ).unwrap();
        assert_eq!(movement, -7_000);
        assert_eq!(
            list_replenishment(&connection).unwrap()[0].suggested_quantity_millis,
            17_000
        );
    }

    #[test]
    fn financial_failure_rolls_back_sale_and_stock() {
        let mut connection = database();
        let id = product(&connection, "Producto", None);
        add_initial_stock(
            &mut connection,
            StockInput {
                product_id: id,
                quantity_millis: 1_000,
                occurred_at: "2026-10-05".into(),
                note: None,
            },
        )
        .unwrap();
        connection
            .execute_batch(
                "CREATE TRIGGER fail_sale_finance BEFORE INSERT ON financial_movements
             WHEN NEW.source_type = 'SALE' BEGIN SELECT RAISE(ABORT, 'forced failure'); END;",
            )
            .unwrap();
        let result = confirm_sale(
            &mut connection,
            OperationInput {
                occurred_at: "2026-10-05".into(),
                payment_method: "CASH".into(),
                items: vec![LineInput {
                    product_id: id,
                    quantity_millis: 1_000,
                    unit_cost_cents: None,
                }],
            },
        );
        assert!(result.is_err());
        assert_eq!(stock_for_product(&connection, id).unwrap(), 1_000);
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM sales", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn migration_preserves_sprint_one_data_without_guessing_payment() {
        let connection = Connection::open_in_memory().unwrap();
        configure(&connection).unwrap();
        connection.execute_batch(MIGRATION_001).unwrap();
        connection.execute(
            "INSERT INTO sales(occurred_at, status, total_cents, total_cost_cents) VALUES ('2026-09-30', 'CONFIRMED', 150000, 100000)", []
        ).unwrap();
        migrate(&connection).unwrap();
        let (sales, payment, movements): (i64, Option<String>, i64) = (
            connection
                .query_row("SELECT COUNT(*) FROM sales", [], |row| row.get(0))
                .unwrap(),
            connection
                .query_row("SELECT payment_method FROM sales LIMIT 1", [], |row| {
                    row.get(0)
                })
                .unwrap(),
            connection
                .query_row(
                    "SELECT COUNT(*) FROM financial_movements WHERE source_type = 'SALE'",
                    [],
                    |row| row.get(0),
                )
                .unwrap(),
        );
        assert_eq!(sales, 1);
        assert_eq!(payment, None);
        assert_eq!(movements, 1);
    }
}
