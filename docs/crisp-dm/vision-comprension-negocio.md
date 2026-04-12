---
titulo: Fase 1 — Comprensión del Negocio
aliases:
  - comprension-negocio
tags:
  - crisp-dm
  - vision
  - plane
relacionado:
  - "[[MOC]]"
estado: activo
---

# Fase 1: Comprensión del Negocio

> [!SUMMARY] Objetivo
> Entender los objetivos y requisitos del proyecto desde una perspectiva de negocio, y convertir este conocimiento en una definición de un problema de minería de datos.

---

## Objetivos de Negocio

- **Definición del problema:** ¿Qué problema de Plane estamos intentando resolver? (ej. predicción de entrega de issues, detección de cuellos de botella).
- **Criterios de éxito:** ¿Qué métricas de negocio indicarán que el modelo es útil? (ej. reducción del 10% en retrasos de ciclos).

---

## Evaluación de la Situación

### Inventario de Recursos
- **Personal:** Expertos en dominio de Plane, científicos de datos.
- **Datos:** Acceso a PostgreSQL, logs de eventos.
- **Hardware/Software:** Infraestructura en la nube, entornos de Python/Rust.

### Requisitos, Supuestos y Restricciones
- **Privacidad:** Cumplimiento con GDPR/SOC2.
- **Seguridad:** Anonimización de datos de workspaces.

---

## Objetivos de Minería de Datos

- **Metas técnicas:** ¿Qué se busca predecir o clasificar?
- **Criterios de éxito técnico:** Precisión, Recall, MAE, etc.

---

## Plan del Proyecto

| Fase                | Duración estimada | Herramientas principales |
| ------------------- | ----------------- | ------------------------- |
| Comprensión Datos   | 1 semana          | SQL, Pandas               |
| Preparación         | 2 semanas         | dbt, Spark                |
| Modelado/Evaluación | 3 semanas         | Scikit-learn, PyTorch     |

---

## 🔗 Navegar

← [[MOC]] | → [[fundamentos-datos]]
