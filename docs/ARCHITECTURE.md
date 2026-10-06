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
- Migration `003_operational_readiness.sql`: auditoría de anulaciones, enlaces a movimientos compensatorios y snapshots de reversión de compras.
- Importación CSV validada dos veces (preview y confirmación) y aplicada en una transacción `IMMEDIATE`.
- Backup local mediante SQLite Online Backup API; restauración validada por `integrity_check` y versión de esquema, con backup preventivo.

La base se crea en el directorio de datos de la aplicación como `despensa-nahuel.sqlite3`. Los importes se guardan en centavos enteros. Las cantidades se guardan en milésimas enteras (`1 UNIT = 1000`; `0,750 kg = 750`) para sumar y restar sin error decimal acumulativo. El stock actual se obtiene sumando el ledger; no existe un campo mutable de stock en productos.

En Windows, Tauri resuelve normalmente ese directorio como `%APPDATA%\com.despensanahuel.desktop`; la ruta efectiva se muestra en Configuración. El archivo persistente principal es `despensa-nahuel.sqlite3`; `-wal` y `-shm`, si aparecen, son auxiliares transitorios y no deben copiarse a mano. Los backups elegidos por el usuario son archivos SQLite portables; las copias preventivas de restauración quedan en `backups` bajo el directorio de datos.

Los medios soportados son `CASH`, `TRANSFER`, `CARD` y `OTHER`. Operaciones de Sprint 1 conservan medio `NULL` y se muestran como “Sin clasificar”; la migration no inventa efectivo histórico. Los movimientos financieros son append-only. La caja esperada deriva del saldo inicial y los movimientos `CASH`; el cierre guarda contado y diferencia. Los conteos físicos guardan esperado, contado, diferencia, costo snapshot y generan `ADJUSTMENT` en el ledger cuando corresponde.

Las anulaciones nunca modifican ni borran movimientos originales. Ventas, gastos y compras reversibles cambian a `CANCELLED` (presentado como **ANULADA**) y agregan compensaciones enlazadas. Para compras se exige evidencia no nula en `purchase_items`, ausencia total de movimientos posteriores por producto, stock exacto esperado y costo actual igual al costo resultante guardado.

## Formato CSV inicial

Cabecera exacta: `name,barcode,category,unit_type,cost,sale_price,initial_stock,min_stock,target_stock`. Es CSV UTF-8 separado por coma; valores argentinos con coma decimal deben ir entre comillas, por ejemplo `"1.250,50"`. Dinero acepta `1250`, `1250,50` y `1.250,50`; un punto sin coma (`1.250`) se rechaza por ambiguo. Cantidades aceptan hasta tres decimales con coma; `UNIT` exige enteros y `WEIGHT` usa milésimas. `barcode`, categoría, costo, stock inicial y niveles de reposición pueden quedar vacíos; `name`, `unit_type` y `sale_price` son obligatorios. Cualquier fila inválida bloquea el lote completo.

## PROPUESTO / FUTURO

- Cloud sólo podría evaluarse como decisión futura; no está incluido ni configurado.
- Importación masiva para actualizar productos existentes.

No hay cloud, autenticación ni API remota.
