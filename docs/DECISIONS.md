# Decisiones

## D001 — Desktop offline-first

**Decisión:** aplicación Windows donde la PC local es autoridad operativa.  
**Motivo:** la operación cotidiana no debe depender de Internet.  
**Consecuencias:** se diseña para persistencia local; cloud no es requisito de operación.

## D002 — Tauri + React/Vite/TypeScript

**Decisión:** usar Tauri 2 como contenedor y React/Vite/TypeScript en frontend.  
**Motivo:** desktop liviano con un flujo moderno de UI tipada.  
**Consecuencias:** no se usa Next.js ni framework UI grande en el bootstrap.

## D003 — SQLite local prevista; cloud opcional

**Decisión:** SQLite será la persistencia local prevista; Supabase podría servir después para backup/sync.  
**Motivo:** operación local confiable sin diseñar cloud prematuramente.  
**Consecuencias:** no hay esquema, plugin SQLite ni Supabase en Sprint 0.

## D004 — POS separado de Gestión

**Decisión:** distinguir los contextos POS/Vender y Gestión/Administrar.  
**Motivo:** la venta prioriza velocidad; administración requiere más detalle.  
**Consecuencias:** futuras pantallas deben respetar esa separación conceptual.

## D005 — Inventario por movimientos

**Decisión:** el stock futuro será auditable mediante movimientos.  
**Motivo:** explicar ajustes y diferencias, no sólo mostrar una cantidad.  
**Consecuencias:** no modelar el dominio únicamente como `producto.stock`.

## D006 — Caja y rentabilidad separadas

**Decisión:** distinguir flujo de dinero de resultado económico.  
**Motivo:** una compra de mercadería reduce caja pero genera stock.  
**Consecuencias:** reportes y modelo contable futuros deben evitar equiparar compra con pérdida.

## D007 — Dinero y cantidades enteras

**Decisión:** guardar dinero en centavos y cantidades en milésimas enteras. `UNIT` exige múltiplos de 1000; `WEIGHT` admite hasta 0,001 kg.
**Motivo:** SQLite opera estas magnitudes sin errores acumulativos de punto flotante.
**Consecuencias:** toda entrada/salida convierte en los bordes de UI; ampliar precisión requerirá una migration explícita.

## D008 — Stock por ledger sin cache inicial

**Decisión:** calcular stock como suma de `inventory_movements`; no guardar `products.stock`.
**Motivo:** el volumen MVP prioriza una única fuente auditable y correcta.
**Consecuencias:** se agregaron índices por producto/fecha; un cache sólo se evaluará con evidencia de rendimiento.

## D009 — Confirmaciones atómicas e historia inmutable

**Decisión:** compras y ventas se preparan en memoria y se persisten directamente como `CONFIRMED` en una transacción `IMMEDIATE`. No se permite vender ni ajustar por debajo de cero.
**Motivo:** evitar borradores abandonados y estados parciales en el MVP de una sola PC.
**Consecuencias:** operaciones confirmadas no se editan; anulaciones/reversiones quedan para un flujo posterior.

## D010 — Costo promedio con costo desconocido

**Decisión:** si el stock previo es cero/negativo o el costo actual es cero, la primera compra conocida fija su costo unitario; en los demás casos se aplica promedio ponderado redondeado al centavo más cercano.
**Motivo:** stock inicial sin costo no debe diluir artificialmente la primera valuación conocida.
**Consecuencias:** el costo se actualiza junto con compra y movimiento en una única transacción.
