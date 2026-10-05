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
- Documentación canónica y guía de agentes.

## Verificado

- ESLint sin errores.
- `tsc -b` sin errores.
- Build de producción Vite generado correctamente.
- Tests Rust: promedio, compra, venta/snapshot, atomicidad, barcode y persistencia end-to-end.
- La aplicación de desarrollo compiló, abrió y creó la base en App Data.
- Build release y paquetes Windows MSI/NSIS generados correctamente.

## Pendiente

- Discovery con Lili.
- Caja, gastos, resultados, reposición, inventario físico y periféricos.
- Flujos de anulación/reversión para operaciones confirmadas.

## Requiere verificación

- Recorrido visual manual completo del escenario operativo; la autorización de control de Windows venció durante esta ejecución.

## Estado remoto

Nada aplicado remotamente. No se creó commit ni se hizo push.
