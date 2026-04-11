---
titulo: Riesgos operacionales — 13 puntos críticos
aliases:
  - riesgos
  - cosas-criticas
  - puntos-criticos
tags:
  - critico
  - riesgos
  - produccion
  - planificacion
relacionado:
  - "[[MOC]]"
  - "[[plan-fases]]"
  - "[[fundamentos-soft-delete]]"
  - "[[dominio-workspace-seed]]"
  - "[[dominio-integraciones]]"
estado: activo
---

# Riesgos operacionales — 13 puntos críticos

> [!DANGER] Leer antes de implementar cualquier feature
> Esta sección recoge los 13 puntos de mayor riesgo operacional del proyecto.
> Ignorarlos produce bugs silenciosos difíciles de detectar en producción.

---

## 1. Bot user — enum `bot_type` en Postgres

El bot user se crea con `is_bot=true` y `bot_type='WORKSPACE_SEED'`.
`bot_type` es un **enum Postgres**.

**Riesgo:** Verificar que el enum `bot_type_enum` en la migración baseline incluye el valor `'WORKSPACE_SEED'` antes de ejecutar el seed job. Si no existe → error en runtime difícil de diagnosticar.

**Ver:** [[dominio-workspace-seed]]

---

## 2. Soft delete en todas las entidades del seed

Todas las entidades creadas por el seed tienen `deleted_at = NULL`.

**Riesgo:** Los queries en rutas que no usen `.active()` devolverán registros eliminados por el usuario y mezclarán datos de demos con datos reales.

**Regla:** Usar siempre `.active()` en queries de rutas. Ver [[fundamentos-soft-delete]].

---

## 3. Ciclos — fechas relativas, no absolutas

Los JSON de `cycles.json` tienen `type: "CURRENT" | "UPCOMING"`, no fechas.

**Riesgo:** Si se guardan fechas del JSON directamente, los ciclos tendrán fechas del pasado y la UI los mostrará como expirados.

**Solución:** Calcular en runtime:

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

---

## 4. `IssueSequence` — tabla separada obligatoria

Por cada issue creado en el seed se debe crear también un `IssueSequence`.

**Riesgo:** Sin esto las issues no son navegables desde el frontend (el `#` de referencia, ej. `WS-1`, no existe).

---

## 5. Orden de inserción en el seed — FKs obligatorias

```
workspace → bot_user → project → states → labels → cycles → modules → issues → views → pages
```

**Riesgo:** Issues referencian states, labels, cycles y modules via FK. Insertar fuera de orden genera violaciones de FK en runtime que abortan el job.

**Ver:** [[dominio-workspace-seed#Paso 3 — El job apalis]]

---

## 6. `ProjectIdentifier` — tabla separada obligatoria

Al crear el proyecto en el seed, crear también la fila en `project_identifiers`:

```rust
project_identifiers::ActiveModel {
    id:           Set(Uuid::new_v4()),
    workspace_id: Set(workspace_id),
    project_id:   Set(real_project_id),
    identifier:   Set(identifier.clone()),
    created_by_id: Set(bot_id),
    ..Default::default()
}.insert(db).await?;
```

**Riesgo:** Sin esto el proyecto no tiene identifier único y el frontend no puede construir las rutas de issues (`WS-1`, `WS-2`…).

---

## 7. `description_html` en issues — insertar verbatim

Las issues del seed tienen HTML rico con imágenes externas (`media.docs.plane.so`), callouts y listas.

**Riesgo:** Transformar o sanitizar el HTML en el backend rompe el renderizado en la UI.

**Regla:** Pasar como `String` directo a SeaORM sin transformación.

---

## 8. `display_filters` y `display_properties` — JSONB exacto

Al crear `ProjectUserProperty` en el seed, usar los mismos defaults que Django (ver `workspace_seed_task.py` líneas ~90-115).

**Riesgo:** El frontend los consume directamente y espera las claves exactas. Claves incorrectas producen errores silenciosos en la UI.

---

## 9. Entidades Django legacy — NO borrar hasta Fase 5

Las entities `django_celery_beat_*`, `django_content_type`, `django_migrations`, `django_session` siguen presentes en la DB mientras Django esté activo.

**Riesgo:** Borrar los archivos `.rs` correspondientes antes de la Fase 5 rompe la compilación cuando esas tablas son referenciadas por FKs en otras entidades.

**Ver:** [[plan-fases#Fase 5 — Shutdown Django completo]]

---

## 10. `apalis_jobs` table — inicializar antes del primer job

```rust
// En main.rs, al arrancar, antes de registrar workers
PostgresStorage::setup(&db).await?;
```

**Riesgo:** El primer intento de push/pull de job falla con "table not found" si este setup no se ejecuta.

**Ver:** [[impl-error-jobs-cron#3. Inicialización de apalis en main.rs]]

---

## 11. Migración incremental — Traefik routing dual (Fases 1–4)

Durante la migración Django y Rust corren en paralelo. Traefik enruta por path prefix, con Rust tomando mayor prioridad.

**Riesgo:** Un error de configuración en Traefik puede hacer que Rust reciba tráfico de endpoints no implementados aún y retorne 404, o que Django reciba todos los requests ignorando a Rust.

**Ver configuración:** [[plan-fases#Routing Traefik durante las Fases 1 4]]

---

## 12. Redis — `fred` v10, API diferente a `redis-rs`

El proyecto usa `fred` v10 (pool nativo async).

**Diferencias críticas vs `redis-rs`:**
- Pipelines: `client.pipeline()` (no `pipe()`)
- Pub/Sub: usar `subscriber_client()` (no el cliente principal)
- **No mezclar con `deadpool-redis`** — rompe el pool

**Ver:** [[vision-stack#Notas sobre librerías críticas]]

---

## 13. Todas las fechas en UTC — `chrono::DateTime<Utc>`

SeaORM + Postgres almacena en UTC. Los seeds calculan fechas de ciclos en UTC.

**Riesgo:** Usar fecha local en lugar de UTC produce ciclos con fechas incorrectas que varían según el servidor donde corre el job.

**Regla:** Usar siempre `chrono::Utc::now()`. El frontend convierte a timezone local en el cliente.

---

## Checklist pre-deploy

Antes de cualquier deploy a producción, verificar:

- [ ] `PostgresStorage::setup(&db).await?` en `main.rs`
- [ ] Enum `bot_type_enum` incluye `'WORKSPACE_SEED'` en el schema
- [ ] Traefik prioridades correctas (Rust=10, Django=5 durante migración)
- [ ] `fred` v10 — no mezclar con `deadpool-redis`
- [ ] Fechas calculadas en UTC en todos los jobs
- [ ] Soft delete `.active()` en todos los queries de rutas públicas

---

## 🔗 Navegar

← [[plan-fases]] | [[MOC]]

**Relacionado:** Workspace Seed: [[dominio-workspace-seed]] | Integraciones: [[dominio-integraciones]] | Soft delete: [[fundamentos-soft-delete]]
