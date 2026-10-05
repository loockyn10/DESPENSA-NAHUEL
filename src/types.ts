export type UnitType = "UNIT" | "WEIGHT";
export type PaymentMethod = "CASH" | "TRANSFER" | "CARD" | "OTHER";

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
  reorderMinMillis: number | null;
  reorderTargetMillis: number | null;
  createdAt: string;
  updatedAt: string;
}

export interface ProductInput {
  name: string;
  barcode: string | null;
  categoryId: number | null;
  unitType: UnitType;
  salePriceCents: number;
  reorderMinMillis: number | null;
  reorderTargetMillis: number | null;
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
  paymentMethod: PaymentMethod | null;
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
  paymentMethod: PaymentMethod | null;
  items: SaleItemSnapshot[];
}

export interface ExpenseCategory { id: number; name: string }
export interface Expense {
  id: number; occurredAt: string; categoryId: number; categoryName: string;
  description: string; amountCents: number; paymentMethod: PaymentMethod; note: string | null;
}
export interface FinancialMovement {
  id: number; occurredAt: string; sourceType: string; sourceId: number | null;
  amountCents: number; direction: "INCOME" | "OUTFLOW"; paymentMethod: PaymentMethod | null;
  description: string; note: string | null;
}
export interface CashSummary {
  sessionId: number | null; businessDate: string; status: "OPEN" | "CLOSED" | null;
  openingCashCents: number; cashIncomeCents: number; cashOutflowCents: number;
  expectedCashCents: number; countedCashCents: number | null; differenceCents: number | null;
}
export interface InventoryCountSummary {
  id: number; occurredAt: string; itemCount: number;
  negativeValueCents: number; positiveValueCents: number;
}
export interface ReplenishmentItem {
  productId: number; productName: string; unitType: UnitType; stockMillis: number;
  reorderMinMillis: number; reorderTargetMillis: number; suggestedQuantityMillis: number;
  currentCostCents: number; estimatedCostCents: number;
}
export interface DashboardSummary {
  startDate: string; endDateExclusive: string; salesCents: number; costOfGoodsCents: number;
  grossProfitCents: number; expensesCents: number; estimatedResultCents: number;
  purchasesCents: number; cashInCents: number; cashOutCents: number; inventoryValueCents: number;
  replenishmentCount: number; negativeInventoryDifferenceCents: number;
  positiveInventoryDifferenceCents: number; unknownPaymentCount: number;
}
