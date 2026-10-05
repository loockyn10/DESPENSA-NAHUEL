# Producto

## Usuario, problema y valor

Lili opera una única despensa desde la PC del negocio. Necesita control simple de ventas, compras, stock, costos, precios, márgenes, gastos, caja y reposición. Propuesta: **“Vendé normalmente y dejá que el sistema te diga qué comprar, cuánto estás ganando y dónde se está yendo la plata.”**

## Principios

- Offline-first; la PC local es la autoridad operativa.
- Dos contextos UX: **POS / Vender** para velocidad y **Gestión / Administrar** para el resto.
- Carga mínima, sin requerir saberes técnicos o contables.
- Caja/flujo de dinero y resultado/rentabilidad son conceptos distintos: comprar mercadería reduce caja y genera stock, no una pérdida automática.

## Modelo conceptual

Circuito MVP: compra → stock y costo → precio → venta → margen → faltantes → reposición → gastos → resultado.

El inventario será auditable por movimientos (compra, venta, ajuste, vencimiento, rotura, consumo y diferencia), no sólo por un campo de stock. Se propone costo promedio ponderado; las ventas conservarán snapshots de precio y costo. Productos futuros: `UNIT` y `WEIGHT`; se prioriza scanner USB, reposición e inventario físico.

## Producto implementado

El circuito disponible es: categoría/producto → stock inicial o compra → costo promedio → venta → descuento de stock → control físico → reposición. El POS permite búsqueda por nombre o código, cantidades por unidad o peso, medio de pago y confirmación atómica. Productos inactivos permanecen en el historial.

Inicio explica el mes actual (o un rango explícito) con ventas, costo histórico vendido, ganancia bruta, gastos y **resultado estimado**. Compras de mercadería y flujo de dinero se muestran por separado para no confundir caja con rentabilidad. Gastos, caja diaria, ajustes manuales justificados, inventarios físicos parciales y lista de reposición están disponibles con lenguaje operativo.
