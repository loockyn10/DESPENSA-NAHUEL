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
