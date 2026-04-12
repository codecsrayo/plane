---
titulo: Fase 5 — Evaluación
aliases:
  - evaluacion
tags:
  - crisp-dm
  - referencia
relacionado:
  - "[[MOC]]"
  - "[[dominio-modelado]]"
estado: activo
---

# Fase 5: Evaluación

> [!SUMMARY] Objetivo
> Evaluar el modelo desde la perspectiva de los objetivos de negocio definidos en la Fase 1.

---

## Evaluación de Resultados

- **Cumplimiento de objetivos:** ¿El modelo ayuda a predecir retrasos con una precisión aceptable para los Project Managers?
- **Impacto:** Se estima que el uso del modelo puede alertar sobre el 80% de los issues que excederán su ciclo.

---

## Revisión del Proceso

> [!IMPORTANT] Lecciones aprendidas
> - La calidad de los datos en `issue_activities` es vital para el modelo.
> - Se detectó un sesgo en workspaces con pocos miembros que debe corregirse.

---

## Determinación de Próximos Pasos

- **Opción A:** Proceder al despliegue como una funcionalidad beta en Plane.
- **Opción B:** Volver a la Fase 3 para incluir datos de integraciones externas (GitHub/Slack).

---

## 🔗 Navegar

← [[dominio-modelado]] | [[MOC]] | → [[plan-despliegue]]
