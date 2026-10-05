import type { UnitType } from "./types";

const ars = new Intl.NumberFormat("es-AR", {
  style: "currency",
  currency: "ARS",
  minimumFractionDigits: 2,
});

export const money = (cents: number) => ars.format(cents / 100);

export function parseMoney(value: string): number | null {
  const clean = value.replace(/[$\s]/g, "");
  if (!clean) return null;
  const normalized = clean.includes(",") ? clean.replace(/\./g, "").replace(",", ".") : clean;
  if (!/^\d+(\.\d{0,2})?$/.test(normalized)) return null;
  const amount = Number(normalized);
  return Number.isSafeInteger(Math.round(amount * 100)) ? Math.round(amount * 100) : null;
}

export function parseQuantity(value: string, unitType: UnitType): number | null {
  const normalized = value.trim().replace(",", ".");
  if (!/^\d+(\.\d{0,3})?$/.test(normalized)) return null;
  const quantity = Number(normalized);
  const millis = Math.round(quantity * 1000);
  if (millis <= 0 || !Number.isSafeInteger(millis)) return null;
  if (unitType === "UNIT" && millis % 1000 !== 0) return null;
  return millis;
}

export function quantity(millis: number, unitType: UnitType): string {
  const value = millis / 1000;
  return new Intl.NumberFormat("es-AR", {
    minimumFractionDigits: unitType === "WEIGHT" ? 3 : 0,
    maximumFractionDigits: 3,
  }).format(value);
}

export function today(): string {
  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export const errorMessage = (error: unknown) =>
  typeof error === "string" ? error : error instanceof Error ? error.message : "Ocurrió un error inesperado.";
