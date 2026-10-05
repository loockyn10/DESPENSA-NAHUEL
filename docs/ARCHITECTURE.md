# Arquitectura

## IMPLEMENTADO / VERIFICADO

- Frontend mínimo: React + Vite + TypeScript.
- Contenedor de escritorio: configuración Tauri 2 y crate Rust mínimo.
- Comandos definidos para desarrollo, lint y builds.
- SQLite local con `rusqlite` embebido y migration versionada `001_initial.sql`.
- Capa de comandos Tauri: React no abre ni modifica SQLite directamente.
- Catálogo, categorías, ledger de inventario, compras, costo promedio, ventas y snapshots históricos.
- Confirmaciones de compra/venta y movimientos relacionados dentro de transacciones SQLite `IMMEDIATE`.
- Migration `002_business_control.sql`: medios de pago, ledger financiero, gastos, caja diaria, conteos físicos y niveles de reposición.
- Ventas, compras y gastos crean su movimiento financiero en la misma transacción que la operación origen.
- Reportes por rango explícito separan flujo financiero de resultado económico; el costo vendido proviene de snapshots de `sale_items`.

La base se crea en el directorio de datos de la aplicación como `despensa-nahuel.sqlite3`. Los importes se guardan en centavos enteros. Las cantidades se guardan en milésimas enteras (`1 UNIT = 1000`; `0,750 kg = 750`) para sumar y restar sin error decimal acumulativo. El stock actual se obtiene sumando el ledger; no existe un campo mutable de stock en productos.

Los medios soportados son `CASH`, `TRANSFER`, `CARD` y `OTHER`. Operaciones de Sprint 1 conservan medio `NULL` y se muestran como “Sin clasificar”; la migration no inventa efectivo histórico. Los movimientos financieros son append-only. La caja esperada deriva del saldo inicial y los movimientos `CASH`; el cierre guarda contado y diferencia. Los conteos físicos guardan esperado, contado, diferencia, costo snapshot y generan `ADJUSTMENT` en el ledger cuando corresponde.

## PROPUESTO / FUTURO

- Supabase podría ser opcional para backup, sincronización o consulta remota; no está incluido ni configurado.
- Anulaciones/reversiones auditables de operaciones confirmadas.

No hay cloud, autenticación ni API remota.
