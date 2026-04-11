---
titulo: Cosas críticas — riesgos operacionales
tags:
  - critico
  - riesgos
  - produccion
relacionado:
  - "[[06-soft-delete]]"
  - "[[09-workspace-seed]]"
  - "[[14-integraciones]]"
---

> [!DANGER] Leer antes de implementar cualquier feature
> Esta sección recoge los 13 puntos de mayor riesgo operacional del proyecto.
> Ignorarlos produce bugs silenciosos difíciles de detectar en producción.

## Cosas críticas a tener en cuenta

### 1. Bot user — enum `bot_type` en Postgres

El bot user se crea con `is_bot=true` y `bot_type='WORKSPACE_SEED'`.
`bot_type` es un enum Postgres. Verificar que el enum `bot_type_enum`
en la migración baseline incluye el valor `'WORKSPACE_SEED'` antes de
ejecutar el seed job — si no existe → error en runtime.

### 2. Soft delete en todas las entidades del seed

Todas las entidades creadas por el seed tienen `deleted_at = NULL`.
Los queries en rutas deben siempre usar `.active()` para no devolver
registros eliminados por el usuario.

### 3. Ciclos — fechas relativas, no absolutas

Los JSON de `cycles.json` tienen `type: "CURRENT" | "UPCOMING"`, no fechas.
Calcular en runtime:

```rust
let now = Utc::now();
let (start_date, end_date) = match cycle_seed.cycle_type.as_str() {
    "CURRENT"  => (now, now + Duration::days(14)),
    "UPCOMING" => {
        let last = cycles::Entity::find()
            .filter(cycles::Column::ProjectId.eq(real_project_id))
            .order_by_desc(cycles::Column::EndDate)
            .one(db).await?;
        match last {
            Some(c) => (c.end_date + Duration::days(1), c.end_date + Duration::days(15)),
            None    => (now + Duration::days(14), now + Duration::days(28)),
        }
    }
    _ => return Err(anyhow::anyhow!("Unknown cycle type: {}", cycle_seed.cycle_type)),
};
```

### 4. `IssueSequence` — tabla separada obligatoria

Por cada issue creado en el seed se debe crear también un `IssueSequence`.
Es lo que genera el `#` de referencia visible en la UI. Sin esto las issues
no son navegables desde el frontend.

### 5. Dependencias en el seed — orden estricto

```
workspace → bot_user → project → states → labels → cycles → modules → issues → views → pages
```

Issues referencian states, labels, cycles y modules via FK. Insertar fuera
de orden genera violaciones de FK en runtime.

### 6. `ProjectIdentifier` — tabla separada obligatoria

Al crear el proyecto en el seed, crear también la fila en `project_identifiers`:

```rust
project_identifiers::ActiveModel {
    id: Set(Uuid::new_v4()),
    workspace_id: Set(workspace_id),
    project_id: Set(real_project_id),
    identifier: Set(identifier.clone()),
    created_by_id: Set(bot_id),
    ..Default::default()
}.insert(db).await?;
```

Sin esto el proyecto no tiene identifier único y el frontend no puede
construir las rutas de issues (`WS-1`, `WS-2`…).

### 7. `description_html` en issues — insertar verbatim

Las issues del seed tienen HTML rico con imágenes externas
(`media.docs.plane.so`), callouts y listas. No transformar ni validar.
Se pasa como `String` directo a SeaORM.

### 8. `display_filters` y `display_properties` — JSONB exacto

Al crear `ProjectUserProperty` en el seed, usar los mismos defaults que
Django (ver `workspace_seed_task.py` líneas ~90-115). El frontend los
consume directamente sin transformación y espera las claves exactas.

### 9. Entidades Django legacy — NO borrar hasta Fase 5

Las entities `django_celery_beat_*`, `django_content_type`,
`django_migrations`, `django_session` siguen presentes en la DB mientras
Django esté activo. No borrar los archivos `.rs` correspondientes hasta
completar la Fase 5 (shutdown Django).

### 10. `apalis_jobs` table — inicializar antes del primer job

```rust
// En main.rs, al arrancar, antes de registrar workers
PostgresStorage::setup(&db).await?;
```

Sin esto el primer intento de push/pull de job falla con "table not found".

### 11. Migración incremental — Traefik routing dual (Fases 1–4)

Durante la migración Django y Rust corren en paralelo. Traefik enruta
por path prefix, con Rust tomando mayor prioridad:

```yaml
# Rust — alta prioridad, toma los endpoints ya migrados
- "traefik.http.routers.api-rust.rule=PathPrefix(`/api/`)"
- "traefik.http.routers.api-rust.priority=10"
# Django — baja prioridad, solo recibe lo que Rust no maneja aún
- "traefik.http.routers.api-django.rule=PathPrefix(`/api/`)"
- "traefik.http.routers.api-django.priority=5"
```

Esto permite mover endpoints uno a uno sin downtime.

### 12. Redis — `fred` v10, API diferente a `redis-rs`

El proyecto usa `fred` v10 (pool nativo async). Los pipelines usan
`client.pipeline()`. Para pub/sub (plane-live sync entre instancias)
usar `subscriber_client`. No mezclar con `deadpool-redis`.

### 13. Todas las fechas en UTC — `chrono::DateTime<Utc>`

SeaORM + Postgres almacena en UTC. Los seeds calculan fechas de ciclos
en UTC. El frontend convierte a timezone local en el cliente.

---

