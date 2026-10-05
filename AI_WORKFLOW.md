# Flujo de trabajo con IA

Idea → definición con arquitecto → sprint pequeño → un agente principal → implementación → tests → documentación → revisión → commit/push sólo tras aprobación → nuevo contexto.

Usar aproximadamente 1–3 prompts sustanciales por contexto y cambiar de contexto cuando cambia la naturaleza de la tarea. El repositorio y sus docs son memoria permanente; los chats son contexto descartable.

Preferir Codex para implementaciones bien especificadas. Preferir Claude para exploración, auditoría, debugging complejo, arquitectura sensible y auth/RLS/seguridad. Sumar un segundo agente sólo si aporta valor real; el trabajo paralelo exige branches o worktrees separados.
