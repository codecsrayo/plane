---
titulo: Siembra Inicial de Datos (Seed)
aliases:
  - siembra-datos
  - seed-data
tags:
  - fundamentos
  - migraciones
  - seed
  - workspace
  - apalis
relacionado:
  - "[[MOC]]"
  - "[[fundamentos-migraciones-estado]]"
  - "[[dominio-workspace-seed]]"
estado: activo
---

# Siembra Inicial de Datos (Seed)

El sistema de Plane requiere una serie de datos predefinidos para funcionar correctamente desde el primer arranque. En la arquitectura Rust, esta siembra se divide en dos niveles: **Estática (Global)** y **Dinámica (Workspace)**.

---

## 1. Siembra Estática (Global)

Se ejecuta automáticamente durante la fase de migración de la base de datos. Estos datos son comunes a toda la instancia y no dependen de la creación de un workspace.

### Implementación: Migración `m007`
La migración `m20240101_000007_seed_data.rs` es la encargada de esta tarea.

- **Tablas afectadas**:
  - `integrations`: Inserta los proveedores base (`github`, `gitlab`, `slack`).
  - `instance_configurations`: Inserta todas las claves de configuración por defecto (Auth, SMTP, AI, etc.).

- **Garantía de Idempotencia**:
  Se utiliza la cláusula `ON CONFLICT (provider) DO NOTHING` e `ON CONFLICT (key) DO NOTHING`. Esto permite que la migración se ejecute de forma segura incluso si los datos ya fueron sembrados previamente por Django.

- **Valores por defecto**:
  Los valores insertados son los "hardcoded defaults". Los overrides mediante variables de entorno se aplican en tiempo de ejecución, no en la base de datos durante la migración.

---

## 2. Siembra de Workspace (Datos Demo)

Se activa asíncronamente cada vez que se crea un nuevo Workspace. Su objetivo es proporcionar una experiencia "out-of-the-box" con proyectos, estados e issues de ejemplo.

### Mecanismo: `WorkspaceSeedJob` (Apalis)
A diferencia de Django (que usa Celery), Rust utiliza **Apalis** para gestionar esta tarea en segundo plano.

- **Disparador**: `POST /api/workspaces/` encola un `WorkspaceSeedJob`.
- **Fuente de datos**: Archivos JSON ubicados en `apps/api/plane/seeds/data/`.
  - En Rust, estos archivos se embeben en el binario usando `include_str!()` para evitar dependencias de rutas de archivos en runtime.
- **Entidades creadas**:
  1. **Bot User**: Un usuario con `is_bot=true` que actúa como creador de los datos.
  2. **Proyecto Demo**: Con su respectivo `ProjectIdentifier`.
  3. **Estados**: (Backlog, Todo, In Progress, Done, Cancelled).
  4. **Ciclos y Módulos**: Calculando fechas relativas al momento de creación.
  5. **Issues**: Incluyendo secuencias, etiquetas y descripciones en HTML.

### Control de Concurrencia
Para evitar condiciones de carrera (race conditions) donde dos workers intenten sembrar el mismo workspace simultáneamente:
- Se utiliza un **Advisory Lock** de Postgres (`pg_try_advisory_xact_lock`) basado en el `workspace_id`.

---

## 📊 Resumen de Responsabilidades

| Tipo de Dato | Momento | Mecanismo | Idempotencia |
| :--- | :--- | :--- | :--- |
| **Integraciones** | Migración Inicial | SeaORM Migration (`m007`) | `ON CONFLICT` |
| **Configuración** | Migración Inicial | SeaORM Migration (`m007`) | `ON CONFLICT` |
| **Proyecto Demo** | Creación Workspace | `WorkspaceSeedJob` (Apalis) | Advisory Lock + Check |
| **Bot del Seed** | Creación Workspace | `WorkspaceSeedJob` (Apalis) | Check persistencia |

---

## Situación Real y Estado de Implementación

> [!IMPORTANT] Estado de la migración
> Actualmente, la **Siembra Estática** (`m007`) está **✅ Completamente Implementada** y se aplica al ejecutar las migraciones de Rust.
>
> La **Siembra de Workspace** está **📝 Diseñada** (ver `[[dominio-workspace-seed]]`) pero su implementación completa en el código Rust (`src/jobs/workspace_seed/`) se encuentra en fase de desarrollo, utilizando los mismos JSONs que la versión de Django para asegurar paridad funcional.

---

## 🔗 Navegar

← [[fundamentos-migraciones-estado]] | [[MOC]] | → [[dominio-workspace-seed]]

---

_`docs/api-rust/fundamentos-siembra-datos.md` · rama `feature/integrations-panel-fix-17593507967815292912`_
