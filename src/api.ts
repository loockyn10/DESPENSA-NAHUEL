import { invoke } from "@tauri-apps/api/core";
import type {
  Category,
  InventoryMovement,
  LineInput,
  OperationSummary,
  Product,
  ProductInput,
  SaleDetail,
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
  confirmPurchase: (occurredAt: string, items: LineInput[]) =>
    invoke<OperationSummary>("confirm_purchase", { input: { occurredAt, items } }),
  listPurchases: () => invoke<OperationSummary[]>("list_purchases"),
  confirmSale: (occurredAt: string, items: LineInput[]) =>
    invoke<OperationSummary>("confirm_sale", { input: { occurredAt, items } }),
  listSales: () => invoke<OperationSummary[]>("list_sales"),
  getSale: (id: number) => invoke<SaleDetail>("get_sale", { id }),
};
