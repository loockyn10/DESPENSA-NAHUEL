export type UnitType = "UNIT" | "WEIGHT";

export interface Category {
  id: number;
  name: string;
}

export interface Product {
  id: number;
  name: string;
  barcode: string | null;
  categoryId: number | null;
  categoryName: string | null;
  unitType: UnitType;
  currentCostCents: number;
  salePriceCents: number;
  active: boolean;
  stockMillis: number;
  createdAt: string;
  updatedAt: string;
}

export interface ProductInput {
  name: string;
  barcode: string | null;
  categoryId: number | null;
  unitType: UnitType;
  salePriceCents: number;
}

export interface LineInput {
  productId: number;
  quantityMillis: number;
  unitCostCents: number | null;
}

export interface OperationSummary {
  id: number;
  occurredAt: string;
  status: string;
  totalCents: number;
  totalCostCents: number | null;
  itemCount: number;
}

export interface InventoryMovement {
  id: number;
  productId: number;
  productName: string;
  quantityMillis: number;
  movementType: string;
  occurredAt: string;
  referenceType: string | null;
  referenceId: number | null;
  note: string | null;
}

export interface SaleItemSnapshot {
  productId: number;
  productName: string;
  quantityMillis: number;
  unitPriceCents: number;
  unitCostCents: number;
  subtotalCents: number;
  totalCostCents: number;
}

export interface SaleDetail {
  id: number;
  occurredAt: string;
  totalCents: number;
  totalCostCents: number;
  items: SaleItemSnapshot[];
}
