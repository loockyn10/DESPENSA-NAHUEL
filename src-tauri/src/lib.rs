mod db;
mod models;

use models::*;
use std::path::PathBuf;
use tauri::{Manager, State};

struct AppState {
    database_path: PathBuf,
}

fn connection(state: &State<'_, AppState>) -> Result<rusqlite::Connection, String> {
    db::open(&state.database_path)
}

#[tauri::command]
fn list_categories(state: State<'_, AppState>) -> Result<Vec<Category>, String> {
    db::list_categories(&connection(&state)?)
}

#[tauri::command]
fn create_category(state: State<'_, AppState>, name: String) -> Result<Category, String> {
    db::create_category(&connection(&state)?, &name)
}

#[tauri::command]
fn update_category(state: State<'_, AppState>, id: i64, name: String) -> Result<Category, String> {
    db::update_category(&connection(&state)?, id, &name)
}

#[tauri::command]
fn list_products(state: State<'_, AppState>) -> Result<Vec<Product>, String> {
    db::list_products(&connection(&state)?)
}

#[tauri::command]
fn create_product(state: State<'_, AppState>, input: ProductInput) -> Result<i64, String> {
    db::create_product(&connection(&state)?, input)
}

#[tauri::command]
fn update_product(state: State<'_, AppState>, id: i64, input: ProductInput) -> Result<(), String> {
    db::update_product(&connection(&state)?, id, input)
}

#[tauri::command]
fn set_product_active(state: State<'_, AppState>, id: i64, active: bool) -> Result<(), String> {
    db::set_product_active(&connection(&state)?, id, active)
}

#[tauri::command]
fn add_initial_stock(state: State<'_, AppState>, input: StockInput) -> Result<(), String> {
    db::add_initial_stock(&mut connection(&state)?, input)
}

#[tauri::command]
fn adjust_stock(state: State<'_, AppState>, input: StockInput) -> Result<(), String> {
    db::adjust_stock(&mut connection(&state)?, input)
}

#[tauri::command]
fn list_movements(
    state: State<'_, AppState>,
    product_id: Option<i64>,
) -> Result<Vec<InventoryMovement>, String> {
    db::list_movements(&connection(&state)?, product_id)
}

#[tauri::command]
fn confirm_purchase(
    state: State<'_, AppState>,
    input: OperationInput,
) -> Result<OperationSummary, String> {
    db::confirm_purchase(&mut connection(&state)?, input)
}

#[tauri::command]
fn list_purchases(state: State<'_, AppState>) -> Result<Vec<OperationSummary>, String> {
    db::list_purchases(&connection(&state)?)
}

#[tauri::command]
fn confirm_sale(
    state: State<'_, AppState>,
    input: OperationInput,
) -> Result<OperationSummary, String> {
    db::confirm_sale(&mut connection(&state)?, input)
}

#[tauri::command]
fn list_sales(state: State<'_, AppState>) -> Result<Vec<OperationSummary>, String> {
    db::list_sales(&connection(&state)?)
}

#[tauri::command]
fn get_sale(state: State<'_, AppState>, id: i64) -> Result<SaleDetail, String> {
    db::get_sale(&connection(&state)?, id)
}

#[tauri::command]
fn list_expense_categories(state: State<'_, AppState>) -> Result<Vec<ExpenseCategory>, String> {
    db::list_expense_categories(&connection(&state)?)
}

#[tauri::command]
fn create_expense_category(
    state: State<'_, AppState>,
    name: String,
) -> Result<ExpenseCategory, String> {
    db::create_expense_category(&connection(&state)?, &name)
}

#[tauri::command]
fn create_expense(state: State<'_, AppState>, input: ExpenseInput) -> Result<Expense, String> {
    db::create_expense(&mut connection(&state)?, input)
}

#[tauri::command]
fn list_expenses(state: State<'_, AppState>) -> Result<Vec<Expense>, String> {
    db::list_expenses(&connection(&state)?)
}

#[tauri::command]
fn list_financial_movements(state: State<'_, AppState>) -> Result<Vec<FinancialMovement>, String> {
    db::list_financial_movements(&connection(&state)?)
}

#[tauri::command]
fn get_cash_summary(
    state: State<'_, AppState>,
    business_date: String,
) -> Result<CashSummary, String> {
    db::get_cash_summary(&connection(&state)?, &business_date)
}

#[tauri::command]
fn open_cash_session(
    state: State<'_, AppState>,
    business_date: String,
    opening_cash_cents: i64,
) -> Result<CashSummary, String> {
    db::open_cash_session(&connection(&state)?, &business_date, opening_cash_cents)
}

#[tauri::command]
fn add_manual_cash_movement(
    state: State<'_, AppState>,
    input: ManualCashInput,
) -> Result<FinancialMovement, String> {
    db::add_manual_cash_movement(&connection(&state)?, input)
}

#[tauri::command]
fn close_cash_session(
    state: State<'_, AppState>,
    business_date: String,
    counted_cash_cents: i64,
    note: Option<String>,
) -> Result<CashSummary, String> {
    db::close_cash_session(
        &mut connection(&state)?,
        &business_date,
        counted_cash_cents,
        note,
    )
}

#[tauri::command]
fn confirm_inventory_count(
    state: State<'_, AppState>,
    input: InventoryCountInput,
) -> Result<InventoryCountSummary, String> {
    db::confirm_inventory_count(&mut connection(&state)?, input)
}

#[tauri::command]
fn list_inventory_counts(state: State<'_, AppState>) -> Result<Vec<InventoryCountSummary>, String> {
    db::list_inventory_counts(&connection(&state)?)
}

#[tauri::command]
fn list_replenishment(state: State<'_, AppState>) -> Result<Vec<ReplenishmentItem>, String> {
    db::list_replenishment(&connection(&state)?)
}

#[tauri::command]
fn dashboard_summary(
    state: State<'_, AppState>,
    start_date: String,
    end_date_exclusive: String,
) -> Result<DashboardSummary, String> {
    db::dashboard_summary(&connection(&state)?, &start_date, &end_date_exclusive)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data = app
                .path()
                .app_data_dir()
                .map_err(|error| format!("No se pudo resolver la carpeta de datos: {error}"))?;
            std::fs::create_dir_all(&app_data)?;
            let database_path = app_data.join("despensa-nahuel.sqlite3");
            let connection = db::open(&database_path)?;
            db::migrate(&connection)?;
            app.manage(AppState { database_path });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_categories,
            create_category,
            update_category,
            list_products,
            create_product,
            update_product,
            set_product_active,
            add_initial_stock,
            adjust_stock,
            list_movements,
            confirm_purchase,
            list_purchases,
            confirm_sale,
            list_sales,
            get_sale,
            list_expense_categories,
            create_expense_category,
            create_expense,
            list_expenses,
            list_financial_movements,
            get_cash_summary,
            open_cash_session,
            add_manual_cash_movement,
            close_cash_session,
            confirm_inventory_count,
            list_inventory_counts,
            list_replenishment,
            dashboard_summary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Despensa Nahuel");
}
