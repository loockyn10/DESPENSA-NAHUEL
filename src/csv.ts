import type { ImportProductRow } from "./types";

const columns = [
  "name", "barcode", "category", "unit_type", "cost", "sale_price",
  "initial_stock", "min_stock", "target_stock",
] as const;

function records(text: string): string[][] {
  const output: string[][] = [];
  let row: string[] = [];
  let field = "";
  let quoted = false;
  for (let index = 0; index < text.length; index += 1) {
    const char = text[index];
    if (quoted) {
      if (char === '"' && text[index + 1] === '"') {
        field += '"';
        index += 1;
      } else if (char === '"') {
        quoted = false;
      } else {
        field += char;
      }
    } else if (char === '"' && field === "") {
      quoted = true;
    } else if (char === ",") {
      row.push(field.trim());
      field = "";
    } else if (char === "\n") {
      row.push(field.trim());
      if (row.some((value) => value !== "")) output.push(row);
      row = [];
      field = "";
    } else if (char !== "\r") {
      field += char;
    }
  }
  if (quoted) throw new Error("El CSV tiene una comilla sin cerrar.");
  row.push(field.trim());
  if (row.some((value) => value !== "")) output.push(row);
  return output;
}

export function parseProductCsv(text: string): ImportProductRow[] {
  const parsed = records(text.replace(/^\uFEFF/, ""));
  const header = parsed.shift()?.map((value) => value.toLowerCase());
  if (!header || columns.some((column, index) => header[index] !== column) || header.length !== columns.length) {
    throw new Error(`La cabecera debe ser exactamente: ${columns.join(",")}`);
  }
  return parsed.map((values, index) => {
    if (values.length !== columns.length) {
      throw new Error(`La fila ${index + 2} tiene ${values.length} columnas; se esperaban ${columns.length}.`);
    }
    return {
      rowNumber: index + 2,
      name: values[0], barcode: values[1], category: values[2], unitType: values[3],
      cost: values[4], salePrice: values[5], initialStock: values[6],
      minStock: values[7], targetStock: values[8],
    };
  });
}
