---
titulo: Fase 2 — Comprensión de los Datos
aliases:
  - comprension-datos
tags:
  - crisp-dm
  - fundamentos
  - datos
relacionado:
  - "[[MOC]]"
  - "[[vision-comprension-negocio]]"
estado: activo
---

# Fase 2: Comprensión de los Datos

> [!SUMMARY] Objetivo
> Familiarizarse con los datos, identificar problemas de calidad y descubrir conocimientos iniciales.

---

## Recolección de Datos Iniciales

Los datos provienen principalmente del schema de PostgreSQL de Plane:
- **Workspaces:** `workspaces`
- **Issues:** `issues`, `issue_activities`
- **Ciclos/Módulos:** `cycles`, `modules`

---

## Descripción de los Datos

| Dataset          | Formato | Cantidad (estimada) | Descripción                                      |
| ---------------- | ------- | ------------------- | ------------------------------------------------ |
| Transaccional    | SQL     | +1M filas           | Estado actual de issues, miembros y proyectos.    |
| Histórico (Logs) | JSONB   | +10M filas          | Historial de cambios en `issue_activities`.      |

---

## Exploración de los Datos

> [!INFO] Hallazgos clave
> - La mayoría de los issues no tienen `due_date`.
> - La actividad de los usuarios se concentra en días laborables.

### Visualizaciones iniciales
- Distribución de estados de issues por workspace.
- Correlación entre número de etiquetas y tiempo de resolución.

---

## Verificación de la Calidad de los Datos

- **Valores nulos:** Alta incidencia en campos opcionales (`description_html`, `estimate_point`).
- **Inconsistencias:** Diferencias entre `updated_at` de un issue y su última actividad registrada.
- **Ruido:** Actividad generada por bots o integraciones automáticas.

---

## 🔗 Navegar

← [[vision-comprension-negocio]] | [[MOC]] | → [[impl-preparacion]]
