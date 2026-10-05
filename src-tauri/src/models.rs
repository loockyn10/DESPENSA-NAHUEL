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
    pub items: Vec<SaleItemSnapshot>,
}
