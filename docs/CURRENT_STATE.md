# Estado actual

## Implementado en repo

- Bootstrap de Tauri 2, React, Vite y TypeScript.
- SQLite local reproducible mediante migration versionada.
- Catálogo de productos `UNIT`/`WEIGHT`, categorías y activación/desactivación.
- Stock por movimientos: inicial, compra, venta y ajuste manual.
- Compras confirmadas con aumento de stock y costo promedio ponderado.
- POS con búsqueda por nombre/barcode, carrito y confirmación de venta.
- Snapshots de nombre, precio y costo en cada item vendido.
- Pantallas Vender, Productos, Categorías, Stock y Compras.
- Medios de pago en ventas, compras y gastos, con históricos previos “Sin clasificar”.
- Ledger financiero auditable y confirmaciones atómicas con dinero, inventario y operación origen.
- Gastos y categorías de gasto extensibles.
- Caja diaria con apertura, efectivo esperado, ajustes justificados, cierre contado y diferencia.
- Dashboard inicial por rango con resultado estimado, flujo, compras, inventario y alertas.
- Inventario físico parcial con snapshots y ajustes de ledger.
- Stock mínimo/objetivo y lista de reposición para `UNIT` y `WEIGHT`.
- Documentación canónica y guía de agentes.

## Verificado

- ESLint sin errores.
- `tsc -b` sin errores.
- Build de producción Vite generado correctamente.
- Tests Rust: promedio, compra, venta/snapshot, atomicidad, barcode, resultado económico, caja, inventario físico, reposición, migración y persistencia end-to-end.
- La aplicación de desarrollo compiló, abrió y creó la base en App Data.
- Build release y paquetes Windows MSI/NSIS generados correctamente.

## Pendiente

- Discovery con Lili.
- Periféricos y anulaciones/reversiones auditables.
- Flujos de anulación/reversión para operaciones confirmadas.

## Requiere verificación

- Recorrido visual manual completo del escenario operativo; la autorización de control de Windows venció durante esta ejecución.

## Estado remoto

Nada aplicado remotamente. No se creó commit ni se hizo push.
