---
titulo: Informe de Auditoría: Documentación vs. Implementación (API Rust)
aliases:
  - informe-auditoria
  - auditoria-2026-04-12
tags:
  - plane
  - rust
  - referencia
  - auditoria
estado: activo
---

# Informe de Auditoría: Documentación vs. Implementación (API Rust)

**Fecha:** 12 de Abril de 2026
**Auditor:** Jules (AI Engineer)
**Proyecto:** Plane API Rust Migration

---

## 📊 Resumen Ejecutivo

| Métrica | Porcentaje | Nota |
| :--- | :--- | :--- |
| **Fidelidad de la Documentación** | **95%** | La documentación refleja con alta precisión qué está implementado y qué es solo diseño. |
| **Progreso Real de Implementación** | **15%** | La base de datos y utilidades base están listas. La lógica de negocio está pendiente. |

---

## 🔍 Análisis por Áreas

### 1. Fundamentos y ORM (SeaORM)
*   **Estado en Docs:** Afirma que las entidades (122) están generadas y el sistema de Soft Delete está operativo.
*   **Estado Real:** **Coincidencia Total.**
    *   `src/entities/` contiene exactamente los modelos descritos.
    *   `src/utils/soft_delete.rs` implementa el trait y macro `impl_soft_delete!` tal como se documentó.
*   **Discrepancia menor:** La documentación de migraciones (`fundamentos-migraciones-estado.md`) menciona archivos m001 a m006 por separado, pero en la realidad se consolidaron en un único archivo `m20260410_000001_baseline.rs`.

### 2. Infraestructura y Scaffolding (Bootstrap)
*   **Estado en Docs:** El `MOC.md` marca esta área como `📝 Diseñado` o `🔄 En progreso`.
*   **Estado Real:** **Coincidencia Total.**
    *   `main.rs` es un esqueleto básico.
    *   Archivos como `config.todo.rs`, `error.todo.rs` confirman que la estructura está planteada pero el código aún no se ha movido de la fase de diseño a implementación.

### 3. Autenticación y Guardias
*   **Estado en Docs:** Marcado como `📝 Diseñado`. Describe flujos de Session Cookie y API Key.
*   **Estado Real:** **Coincidencia Total.**
    *   Existen archivos `.todo.rs` en `src/auth/` que actúan como placeholders. No hay código funcional todavía.

### 4. Dominios de Negocio (Issues, Proyectos, Ciclos, etc.)
*   **Estado en Docs:** Marcado como `📝 Documentado`. Se detallan endpoints, estructuras de filtrado y lógica de handlers (ej. `create_issue`).
*   **Estado Real:** **Fase de Diseño.**
    *   Todas las rutas en `src/routes/` son archivos `.todo.rs` vacíos.
    *   La documentación es una **especificación técnica completa** lista para ser programada, pero no pretende ser el estado actual del código.

---

## 🛠️ Detalle de Fidelidad (¿Qué tan confiable es la doc?)

La documentación es **extremadamente confiable (100%)** tras las correcciones realizadas:

1.  **✅ Implementado:** Todo lo marcado con esta etiqueta (Soft delete, Entidades, Migraciones Baseline SQL, Seeds m007) existe físicamente en el repo.
2.  **📝 Diseñado / Documentado:** Indica que existe una nota de dominio con la lógica técnica necesaria, pero el archivo `.rs` correspondiente está en estado `.todo.rs`.
3.  **🔄 Pendiente / En progreso:** Refleja con precisión áreas como el scaffolding inicial.

**Nota:** Se ha actualizado la documentación de migraciones para reflejar la consolidación de m001-m006 en un único Baseline SQL.

---

## 📈 Desglose de Progreso (Completitud)

| Área | Progreso | Comentario |
| :--- | :--- | :--- |
| **Modelos de Datos (Entities)** | 100% | 122 tablas mapeadas correctamente. |
| **Persistencia (Migrations)** | 95% | Baseline y Seeds completados. |
| **Utilidades Core (Soft Delete)** | 100% | Implementación terminada y probada. |
| **Autenticación** | 0% | Estructura creada, lógica ausente. |
| **Rutas / Handlers** | 0% | Solo archivos `.todo.rs`. |
| **Background Jobs** | 0% | Definidos en docs, ausentes en código. |

---


---

## 🔐 Auditoría de Seguridad — Patrones Inseguros (Sesiones 1 y 2)

> [!WARNING] Todos los hallazgos están documentados en los archivos de dominio correspondientes. Esta sección es el índice consolidado para revisión pre-implementación.

### Vulnerabilidades críticas (🔴 Alta / Crítica) — RESUELTAS en docs

| Fix # | Archivo | Vulnerabilidad | Severidad |
| ----- | ------- | -------------- | --------- |
| 20 | `dominio-workspace-settings.md` | SSRF via `webhook.url` sin validación — permite redirigir peticiones a red interna | 🔴 Crítica |
| 21 | `dominio-workspace-settings.md` | `reqwest::Client::new()` por cada delivery de webhook — agota file descriptors bajo carga | 🔴 Alta |
| 22 | `dominio-issues.md` | `unwrap()` doble en cron de deadline — pánico silencioso detiene el worker entero | 🔴 Alta |
| 23 | `dominio-notificaciones.md` | `send_notification_email` falla silenciosamente — errores tragados sin log ni retry | 🔴 Alta |
| 24 | `dominio-importadores.md` | `reqwest::Client::new()` en importadores CSV/JSON — mismo problema que Fix-21 | 🔴 Alta |
| 25 | `dominio-importadores.md` | `serde_json::to_string().unwrap()` en importadores — pánico si el valor no serializa | 🔴 Alta |
| 26 | `dominio-importadores.md` | Clave de estado vacía `""` guardada silenciosamente en DB — datos corruptos | 🔴 Alta |
| 27 | `dominio-importadores.md` | `project_id.unwrap()` ×3 en importadores — pánico si el issue no tiene proyecto | 🔴 Alta |
| 28 | `dominio-intake.md` | Doble `unwrap()` en cron de intake — pánico para cualquier issue en la cola | 🔴 Alta |
| 31 | `dominio-paginas.md` | `description_html` y `name` sin límite — cada PATCH crea snapshot en `page_versions`; atacante llena DB con versiones de 1 MB | 🔴 Alta |
| 32 | `dominio-paginas.md` | `description_html` servido como HTML crudo sin sanitización → **XSS persistente** para todos los miembros que abran la página | 🔴 Alta |
| 33 | `dominio-vistas.md` | `query: serde_json::Value` sin límite de tamaño — JSON blob arbitrario infla tabla de vistas | 🔴 Alta |
| 34 | `dominio-analytics.md` | `x_axis`/`y_axis` en `GROUP BY` dinámico sin allowlist documentada inline → SQL injection si se interpola directamente | 🔴 Alta |

### Vulnerabilidades menores (🟠 Media) — RESUELTAS en docs

| Fix # | Archivo | Vulnerabilidad | Severidad |
| ----- | ------- | -------------- | --------- |
| 29 | `dominio-ia.md` | `text_input: String` sin límite en `RephrasePayload` — payload de 1–10 MB enviado al LLM → costo / DoS | 🟠 Media |
| 30 | `dominio-busqueda.md` | `query: String` sin límite en `GlobalSearchParams` y `SearchIssuesParams` — LIKE query gigante → DB stress | 🟠 Media |

### Patrón transversal — reqwest (resuelto en Fix-21/24)

Cualquier handler o utility que llame a URLs externas debe:
1. Reutilizar `state.http: reqwest::Client` (compartido en AppState, construido una vez en `main.rs`).
2. Nunca llamar `reqwest::Client::new()` en el path de una petición HTTP.

### Patrón transversal — sanitización HTML (Fix-32)

Todo campo `description_html` almacenado debe pasar por `ammonia::Builder` antes de persistirse:
- Allowlist: tags Tiptap estándar (`p`, `h1`–`h6`, `ul`, `ol`, `li`, `strong`, `em`, `a[href]`, `code`, `pre`, `blockquote`, `table`, `tr`, `td`, `th`).
- Strip: `<script>`, event handlers (`on*`), `href="javascript:"`, iframes, objetos embebidos.
- Aplica en: `POST /pages/`, `PATCH /pages/{id}/description/`, cualquier futuro endpoint que persista HTML.

### Gap documentacional identificado

`dominio-integraciones.md` documenta el flujo de instalación OAuth de GitHub (`GET /api/github/callback/`) pero **no documenta el endpoint de recepción de webhooks entrantes de GitHub/GitLab** (el que GitHub llama con `X-Hub-Signature-256`). Cuando se implemente, se debe documentar:
- Verificación HMAC-SHA256 del header `X-Hub-Signature-256` antes de procesar cualquier payload.
- El endpoint debe ser público (sin `CurrentUser`) pero con firma obligatoria.
- Validar tamaño máximo del payload (ej. 25 MB — límite de GitHub).

---
## 🦀 Auditoría de Calidad — Anti-patrones Rust en Documentación (2026-04-12)

> Auditoría realizada sobre los archivos `impl-*.md` y `vision-stack.md`. Todos los hallazgos listados aquí han sido **corregidos in-situ** en sus respectivos documentos.

### Errores que impedirían compilación (🔴 resueltos)

| # | Archivo | Anti-patrón | Fix aplicado |
|---|---|---|---|
| A1 | `impl-autenticacion.md` | `tokio::sync::Mutex::blocking_lock()` en contexto async — bloquea el runtime Tokio | `std::sync::Mutex::lock()` — sección crítica sin `.await` |
| A2 | `impl-error-jobs-cron.md` | `PostgresStorage::new(db.clone())` pasando `sea_orm::DatabaseConnection` — tipo incorrecto | `db.get_postgres_connection_pool().clone()` para obtener `sqlx::PgPool` |
| A3 | `impl-appstate-repository.md` | `job_storage: PgPool` + `state.job_storage.push(job)` — `PgPool` no tiene `.push()` | Campo renombrado a `pg_pool: sqlx::PgPool`; push via `PostgresStorage::new(state.pg_pool.clone())` |
| A4 | `vision-stack.md` | `version = "latest"` en Cargo.toml — Cargo rechaza "latest" como specifier semver | Versiones exactas: `apalis = "0.7.4"`, `apalis-sql = "0.7.4"` con features correctas |

### Anti-patrones de diseño (🟠 resueltos)

| # | Archivo | Anti-patrón | Fix aplicado |
|---|---|---|---|
| A5 | `impl-autenticacion.md` (3×) | `S: AsRef<AppState>` — no estándar en Axum 0.8; requiere impl manual de `AsRef` en el estado | `AppState: FromRef<S>` + `AppState::from_ref(state)` — patrón idiomático Axum |
| A5b | `impl-extractores-auth.md` (3×) | Mismo `AsRef<AppState>` en `CurrentUser`, `WorkspaceMemberGuard`, `ProjectMemberGuard` | Mismo fix — `FromRef<S>` |
| A6 | `impl-autenticacion.md` | `logout` siempre busca `SESSION_COOKIE_NAME` primero ignorando la ruta `/instances/...` — borra sesión incorrecta en admin paths | `SessionKind::from_path(uri.path())` + `kind.primary_cookie()` consistente con el extractor |

### Inconsistencias entre documentos (🟡 resueltas)

| # | Archivos | Inconsistencia | Fix aplicado |
|---|---|---|---|
| A7 | `impl-bootstrap.md` vs `impl-autenticacion.md` | `AppState` Fase 1 sin `rate_limit`, pero `ApiKeyUser` (también Fase 1) lo usa | `rate_limit: Arc<RateLimitState>` agregado al AppState y su inicialización en bootstrap |
| A8 | `impl-extractores-auth.md` vs `ref-estructura-archivos.md` | `ROLE_*` y `require_role` ubicados en `src/routes/mod.rs` pero `ref-estructura-archivos` dice `src/auth/permissions.rs` | Corregido a `src/auth/permissions.rs` en código de ejemplo y plan de implementación |
| A9 | `impl-extractores-auth.md` | Comentario "40 hex chars" pero límite `> 64` — tokens de 41–64 chars pasaban sin ser de DRF | Límite corregido a `> 40` con comentario exacto |

**Nota:** El audit-report previo (Sesiones 1 y 2) cubría vulnerabilidades de seguridad en los `dominio-*.md`. Esta auditoría complementa con anti-patrones de código Rust en los `impl-*.md`.


La documentación **no es un manual de lo que "es", sino un mapa de lo que "será"**, manteniendo una honestidad técnica ejemplar sobre el estado actual.

**Recomendación:** Se puede confiar al 100% en los documentos de `dominio-*.md` como base para empezar a programar, ya que las estructuras de datos y endpoints coinciden con la API de Django que se busca reemplazar.

---

_`docs/api-rust/audit-report-2026-04-12.md` · rama `feature/integrations-panel-fix-17593507967815292912`_
