# Arquitectura

## IMPLEMENTADO / VERIFICADO

- Frontend mínimo: React + Vite + TypeScript.
- Contenedor de escritorio: configuración Tauri 2 y crate Rust mínimo.
- Comandos definidos para desarrollo, lint y builds.

## PROPUESTO / FUTURO

- SQLite como persistencia local y una capa local de persistencia, a diseñar en un sprint propio.
- Supabase podría ser opcional para backup, sincronización o consulta remota; no está incluido ni configurado.
- Módulos de dominio: POS, catálogo, stock/movimientos, compras/costos, gastos, resultados y reposición.

No hay base de datos, esquema SQL, cloud, autenticación ni API de negocio implementados.
