---
titulo: Fase 4 — Modelado
aliases:
  - modelado
tags:
  - crisp-dm
  - dominio
relacionado:
  - "[[MOC]]"
  - "[[impl-preparacion]]"
estado: activo
---

# Fase 4: Modelado

> [!SUMMARY] Objetivo
> Seleccionar y aplicar diversas técnicas de modelado y calibrar sus parámetros.

---

## Selección de Técnicas de Modelado

- **Tarea:** Predicción de tiempo de resolución (Regresión).
- **Modelos propuestos:**
  1. XGBoost (Baseline).
  2. Random Forest.
  3. Redes Neuronales (MLP).

---

## Diseño del Plan de Prueba

- **Partición:** 80% entrenamiento, 20% prueba (Split por tiempo para evitar data leakage).
- **Validación:** K-fold cross-validation (K=5).

---

## Construcción del Modelo

### Parámetros iniciales (XGBoost)
```python
params = {
    'learning_rate': 0.1,
    'max_depth': 6,
    'n_estimators': 1000
}
```

---

## Evaluación del Modelo (Técnica)

| Modelo        | RMSE | MAE  | R²   |
| ------------- | ---- | ---- | ---- |
| XGBoost       | 12.5 | 8.2  | 0.75 |
| Random Forest | 14.1 | 9.5  | 0.68 |

---

## 🔗 Navegar

← [[impl-preparacion]] | [[MOC]] | → [[ref-evaluacion]]
