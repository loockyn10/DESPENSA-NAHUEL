# Despensa Nahuel — guía de agentes

## Antes de modificar

1. Leer este archivo y la documentación relevante de `docs/`.
2. Ejecutar `git status`.
3. Inspeccionar el código real relacionado.
4. Comprobar nombres, versiones y APIs reales antes de asumir.

## Reglas

- El repositorio actual gana sobre recuerdos de chats; los docs expresan intención y decisiones.
- Si código y docs se contradicen, reportarlo; no resolverlo silenciosamente.
- No inventar tablas, funciones, rutas ni APIs, ni cambiar la arquitectura sin documentarlo.
- Mantener seguridad futura: el frontend no es autoridad de seguridad y no incluir secretos.
- Las migraciones aplicadas no se editan; se agregan nuevas migraciones.
- No ejecutar cambios remotos, commit ni push sin autorización explícita.
- Mantener el scope del sprint. Mejoras independientes van a `docs/TASKS.md`.

## Lectura por tarea

| Tarea | Leer además |
| --- | --- |
| Producto/UX | `PROJECT_CONTEXT.md`, `PRODUCT.md` |
| Arquitectura/persistencia | `ARCHITECTURE.md`, `DECISIONS.md` |
| Planificación | `ROADMAP.md`, `TASKS.md`, `CURRENT_STATE.md` |
| Implementación | `CURRENT_STATE.md` y los docs de su área |

