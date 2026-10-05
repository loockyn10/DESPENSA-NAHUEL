# Arquitectura

## IMPLEMENTADO / VERIFICADO

- Frontend mínimo: React + Vite + TypeScript.
- Contenedor de escritorio: configuración Tauri 2 y crate Rust mínimo.
- Comandos definidos para desarrollo, lint y builds.
- SQLite local con `rusqlite` embebido y migration versionada `001_initial.sql`.
- Capa de comandos Tauri: React no abre ni modifica SQLite directamente.
- Catálogo, categorías, ledger de inventario, compras, costo promedio, ventas y snapshots históricos.
- Confirmaciones de compra/venta y movimientos relacionados dentro de transacciones SQLite `IMMEDIATE`.

La base se crea en el directorio de datos de la aplicación como `despensa-nahuel.sqlite3`. Los importes se guardan en centavos enteros. Las cantidades se guardan en milésimas enteras (`1 UNIT = 1000`; `0,750 kg = 750`) para sumar y restar sin error decimal acumulativo. El stock actual se obtiene sumando el ledger; no existe un campo mutable de stock en productos.

## PROPUESTO / FUTURO

- Supabase podría ser opcional para backup, sincronización o consulta remota; no está incluido ni configurado.
- Módulos de dominio posteriores: gastos, caja, resultados y reposición.

No hay cloud, autenticación ni API remota.
