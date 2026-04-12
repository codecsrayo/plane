---
titulo: Fase 6 — Despliegue
aliases:
  - despliegue
tags:
  - crisp-dm
  - plan
relacionado:
  - "[[MOC]]"
  - "[[ref-evaluacion]]"
estado: activo
---

# Fase 6: Despliegue

> [!SUMMARY] Objetivo
> Integrar el modelo en el flujo de trabajo de Plane y planificar su mantenimiento.

---

## Plan de Despliegue

- **Integración:** El modelo se servirá a través de un nuevo microservicio de Analytics o integrado en el binario de Rust como un worker de `apalis`.
- **Interfaz:** Los resultados se mostrarán en el panel de "Analytics" de cada proyecto.

---

## Plan de Monitoreo y Mantenimiento

- **Frecuencia de re-entrenamiento:** Mensual, utilizando los nuevos datos recolectados.
- **Métricas de degradación:** Si el RMSE sube por encima de 20, disparar una alerta de re-entrenamiento manual.

---

## Informe Final

- Resumen de hallazgos.
- Beneficios económicos y operativos esperados.

---

## Revisión Final del Proyecto

- Cierre de la iteración.
- Archivo de datasets y código de modelado.

---

## 🔗 Navegar

← [[ref-evaluacion]] | [[MOC]]
