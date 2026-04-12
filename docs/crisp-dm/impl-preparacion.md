---
titulo: Fase 3 — Preparación de los Datos
aliases:
  - preparacion-datos
tags:
  - crisp-dm
  - implementacion
relacionado:
  - "[[MOC]]"
  - "[[fundamentos-datos]]"
estado: activo
---

# Fase 3: Preparación de los Datos

> [!SUMMARY] Objetivo
> Seleccionar, limpiar y transformar los datos brutos en el dataset final que se alimentará a los modelos.

---

## Selección de Datos

- **Criterios de inclusión:** Issues creados en los últimos 12 meses con al menos un cambio de estado.
- **Atributos eliminados:** `description_html` (por privacidad y ruido), `attachments` (solo se mantiene el conteo).

---

## Limpieza de Datos

1. **Imputación de nulos:**
   - `priority`: asignar "none" si está vacío.
   - `estimate_point`: asignar 0 o el promedio del proyecto.
2. **Outliers:** Eliminar issues con tiempos de resolución superiores a 2 años (posibles abandonos).

---

## Construcción de Datos (Feature Engineering)

Se generan nuevas variables a partir de los datos existentes:
- `lead_time`: `completed_at` - `created_at`.
- `cycle_efficiency`: tiempo activo vs tiempo en espera.
- `assignee_load`: número de issues abiertos al momento de la creación.

---

## Integración de Datos

- **Merge:** Combinar `issues` con `issue_activities` para aplanar el historial.
- **Agregación:** Resumir métricas a nivel de `project_id` y `workspace_id`.

---

## Formateo de Datos

- Conversión de timestamps a UTC.
- Encoding categórico para `priority` y `state_group`.

---

## 🔗 Navegar

← [[fundamentos-datos]] | [[MOC]] | → [[dominio-modelado]]
