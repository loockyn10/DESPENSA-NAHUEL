import { invoke } from "@tauri-apps/api/core";
import type {
  Category,
  InventoryMovement,
  LineInput,
  OperationSummary,
  Product,
  ProductInput,
  SaleDetail,
  PaymentMethod,
  ExpenseCategory,
  Expense,
  FinancialMovement,
  CashSummary,
  InventoryCountSummary,
  ReplenishmentItem,
  DashboardSummary,
} from "./types";

export const api = {
  listCategories: () => invoke<Category[]>("list_categories"),
  createCategory: (name: string) => invoke<Category>("create_category", { name }),
  updateCategory: (id: number, name: string) => invoke<Category>("update_category", { id, name }),
  listProducts: () => invoke<Product[]>("list_products"),
  createProduct: (input: ProductInput) => invoke<number>("create_product", { input }),
  updateProduct: (id: number, input: ProductInput) => invoke<void>("update_product", { id, input }),
  setProductActive: (id: number, active: boolean) => invoke<void>("set_product_active", { id, active }),
  addInitialStock: (productId: number, quantityMillis: number, occurredAt: string, note: string | null) =>
    invoke<void>("add_initial_stock", { input: { productId, quantityMillis, occurredAt, note } }),
  adjustStock: (productId: number, quantityMillis: number, occurredAt: string, note: string | null) =>
    invoke<void>("adjust_stock", { input: { productId, quantityMillis, occurredAt, note } }),
  listMovements: (productId: number | null) =>
    invoke<InventoryMovement[]>("list_movements", { productId }),
  confirmPurchase: (occurredAt: string, paymentMethod: PaymentMethod, items: LineInput[]) =>
    invoke<OperationSummary>("confirm_purchase", { input: { occurredAt, paymentMethod, items } }),
  listPurchases: () => invoke<OperationSummary[]>("list_purchases"),
  confirmSale: (occurredAt: string, paymentMethod: PaymentMethod, items: LineInput[]) =>
    invoke<OperationSummary>("confirm_sale", { input: { occurredAt, paymentMethod, items } }),
  listSales: () => invoke<OperationSummary[]>("list_sales"),
  getSale: (id: number) => invoke<SaleDetail>("get_sale", { id }),
  listExpenseCategories: () => invoke<ExpenseCategory[]>("list_expense_categories"),
  createExpenseCategory: (name: string) => invoke<ExpenseCategory>("create_expense_category", { name }),
  createExpense: (input: { occurredAt: string; categoryId: number; description: string; amountCents: number; paymentMethod: PaymentMethod; note: string | null }) =>
    invoke<Expense>("create_expense", { input }),
  listExpenses: () => invoke<Expense[]>("list_expenses"),
  listFinancialMovements: () => invoke<FinancialMovement[]>("list_financial_movements"),
  getCashSummary: (businessDate: string) => invoke<CashSummary>("get_cash_summary", { businessDate }),
  openCashSession: (businessDate: string, openingCashCents: number) => invoke<CashSummary>("open_cash_session", { businessDate, openingCashCents }),
  addManualCashMovement: (input: { occurredAt: string; amountCents: number; direction: "INCOME" | "OUTFLOW"; note: string }) =>
    invoke<FinancialMovement>("add_manual_cash_movement", { input }),
  closeCashSession: (businessDate: string, countedCashCents: number, note: string | null) =>
    invoke<CashSummary>("close_cash_session", { businessDate, countedCashCents, note }),
  confirmInventoryCount: (input: { occurredAt: string; note: string | null; items: { productId: number; countedQuantityMillis: number }[] }) =>
    invoke<InventoryCountSummary>("confirm_inventory_count", { input }),
  listInventoryCounts: () => invoke<InventoryCountSummary[]>("list_inventory_counts"),
  listReplenishment: () => invoke<ReplenishmentItem[]>("list_replenishment"),
  dashboardSummary: (startDate: string, endDateExclusive: string) =>
    invoke<DashboardSummary>("dashboard_summary", { startDate, endDateExclusive }),
};
