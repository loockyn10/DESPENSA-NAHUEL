use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub barcode: Option<String>,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub unit_type: String,
    pub current_cost_cents: i64,
    pub sale_price_cents: i64,
    pub active: bool,
    pub stock_millis: i64,
    pub reorder_min_millis: Option<i64>,
    pub reorder_target_millis: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductInput {
    pub name: String,
    pub barcode: Option<String>,
    pub category_id: Option<i64>,
    pub unit_type: String,
    pub sale_price_cents: i64,
    pub reorder_min_millis: Option<i64>,
    pub reorder_target_millis: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockInput {
    pub product_id: i64,
    pub quantity_millis: i64,
    pub occurred_at: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryMovement {
    pub id: i64,
    pub product_id: i64,
    pub product_name: String,
    pub quantity_millis: i64,
    pub movement_type: String,
    pub occurred_at: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineInput {
    pub product_id: i64,
    pub quantity_millis: i64,
    pub unit_cost_cents: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationInput {
    pub occurred_at: String,
    pub payment_method: String,
    pub items: Vec<LineInput>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationSummary {
    pub id: i64,
    pub occurred_at: String,
    pub status: String,
    pub total_cents: i64,
    pub total_cost_cents: Option<i64>,
    pub item_count: i64,
    pub payment_method: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleItemSnapshot {
    pub product_id: i64,
    pub product_name: String,
    pub quantity_millis: i64,
    pub unit_price_cents: i64,
    pub unit_cost_cents: i64,
    pub subtotal_cents: i64,
    pub total_cost_cents: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleDetail {
    pub id: i64,
    pub occurred_at: String,
    pub total_cents: i64,
    pub total_cost_cents: i64,
    pub payment_method: Option<String>,
    pub items: Vec<SaleItemSnapshot>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseCategory {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseInput {
    pub occurred_at: String,
    pub category_id: i64,
    pub description: String,
    pub amount_cents: i64,
    pub payment_method: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Expense {
    pub id: i64,
    pub occurred_at: String,
    pub category_id: i64,
    pub category_name: String,
    pub description: String,
    pub amount_cents: i64,
    pub payment_method: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialMovement {
    pub id: i64,
    pub occurred_at: String,
    pub source_type: String,
    pub source_id: Option<i64>,
    pub amount_cents: i64,
    pub direction: String,
    pub payment_method: Option<String>,
    pub description: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualCashInput {
    pub occurred_at: String,
    pub amount_cents: i64,
    pub direction: String,
    pub note: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CashSummary {
    pub session_id: Option<i64>,
    pub business_date: String,
    pub status: Option<String>,
    pub opening_cash_cents: i64,
    pub cash_income_cents: i64,
    pub cash_outflow_cents: i64,
    pub expected_cash_cents: i64,
    pub counted_cash_cents: Option<i64>,
    pub difference_cents: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryCountLineInput {
    pub product_id: i64,
    pub counted_quantity_millis: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryCountInput {
    pub occurred_at: String,
    pub note: Option<String>,
    pub items: Vec<InventoryCountLineInput>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryCountSummary {
    pub id: i64,
    pub occurred_at: String,
    pub item_count: i64,
    pub negative_value_cents: i64,
    pub positive_value_cents: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplenishmentItem {
    pub product_id: i64,
    pub product_name: String,
    pub unit_type: String,
    pub stock_millis: i64,
    pub reorder_min_millis: i64,
    pub reorder_target_millis: i64,
    pub suggested_quantity_millis: i64,
    pub current_cost_cents: i64,
    pub estimated_cost_cents: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub start_date: String,
    pub end_date_exclusive: String,
    pub sales_cents: i64,
    pub cost_of_goods_cents: i64,
    pub gross_profit_cents: i64,
    pub expenses_cents: i64,
    pub estimated_result_cents: i64,
    pub purchases_cents: i64,
    pub cash_in_cents: i64,
    pub cash_out_cents: i64,
    pub inventory_value_cents: i64,
    pub replenishment_count: i64,
    pub negative_inventory_difference_cents: i64,
    pub positive_inventory_difference_cents: i64,
    pub unknown_payment_count: i64,
}
