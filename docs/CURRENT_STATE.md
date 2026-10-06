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
- Anulación auditable de ventas y gastos mediante movimientos compensatorios.
- Anulación conservadora de compras nuevas sólo cuando stock/costo pueden restaurarse con prueba suficiente.
- Importación inicial CSV con plantilla, preview, errores por fila, duplicados y transacción única.
- Flujo de scanner keyboard-wedge dedicado en POS, incluido pedido manual de peso.
- Backup/restauración local con validación, copia preventiva y datos de instalación visibles.
- Protección central contra doble ejecución de acciones asíncronas sensibles.

## Verificado

- ESLint sin errores.
- `tsc -b` sin errores.
- Build de producción Vite generado correctamente.
- Tests Rust: 19 casos sobre promedio, operaciones, anulaciones, compra reversible/no reversible, CSV, backup/restore inválido, migración y persistencia.
- La aplicación de desarrollo compiló, abrió y creó la base en App Data.
- Build release y paquetes Windows MSI/NSIS generados correctamente.

## Pendiente

- Discovery con Lili.
- Discovery del formato exportable del programa actual de Lili.
- Datos reales de impresora/tickets y prueba física del lector.

## Requiere verificación

- Recorrido visual manual completo con datos reales, lector físico e instalador Windows.

## Estado remoto

Nada aplicado remotamente. No se creó commit ni se hizo push.
