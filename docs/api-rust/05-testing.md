---
titulo: Testing y estructura del proyecto
tags:
  - testing
  - bruno
  - axum-test
  - swagger
relacionado:
  - "[[00-README]]"
  - "[[13-estructura-archivos]]"
---

## Testing de la API

### Documentación interactiva: utoipa + Swagger UI

`utoipa` genera el spec OpenAPI 3.x a partir de macros Rust. Se monta en Axum:

```toml
# Cargo.toml
utoipa = { version = "4", features = ["axum_extras", "uuid", "chrono"] }
utoipa-swagger-ui = { version = "6", features = ["axum"] }
```

```rust
// main.rs — montar Swagger UI en /docs
use utoipa_swagger_ui::SwaggerUi;

let app = Router::new()
    .merge(SwaggerUi::new("/docs")
        .url("/api-docs/openapi.json", ApiDoc::openapi()));
```

Acceder a `http://localhost:8000/docs` para explorar y probar endpoints manualmente.

### Tests automatizados en Rust

**Opción recomendada: `axum-test`**

Permite hacer requests HTTP directamente al router de Axum sin levantar un
servidor TCP real. Ideal para tests de integración rápidos:

```toml
[dev-dependencies]
axum-test = "14"
tokio = { version = "1", features = ["full"] }
```

```rust
#[tokio::test]
async fn test_get_issues() {
    let app = build_app(test_db_state()).await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .get("/api/workspaces/my-ws/projects/1/issues/")
        .add_header("Authorization", "Bearer test-token")
        .await;

    response.assert_status_ok();
    response.assert_json_contains(&json!({ "count": 0 }));
}
```

**Alternativa ligera: `httpc-test`**

Más simple, sin estado entre requests. Útil para smoke tests rápidos:

```toml
[dev-dependencies]
httpc-test = "0.1"
```

```rust
#[tokio::test]
async fn test_health() -> httpc_test::Result<()> {
    let hc = httpc_test::new_client("http://localhost:8000")?;
    let res = hc.do_get("/api/health/").await?;
    res.print().await?;
    Ok(())
}
```

### Cliente externo: Bruno (recomendado sobre Postman/Insomnia)

Bruno es open source, almacena las colecciones como archivos en el repo (no
en la nube), y funciona offline. Las colecciones viven en `apps/api_rust/tests/bruno/`.

```bash
# Instalar Bruno
brew install bruno  # macOS
# o descargar desde https://www.usebruno.com/

# Estructura de colección en el repo
apps/api_rust/tests/bruno/
├── bruno.json
├── environments/
│   ├── local.bru
│   └── staging.bru
├── health/
│   └── get_health.bru
├── issues/
│   ├── list_issues.bru
│   └── create_issue.bru
└── workspaces/
    └── list_workspaces.bru
```

**Comparativa de clientes externos:**

| Cliente    | Open Source | Archivos en git      | Offline | CI/CD          |
| ---------- | ----------- | -------------------- | ------- | -------------- |
| **Bruno**  | ✅          | ✅ archivos `.bru`   | ✅      | ✅ `bruno run` |
| Hoppscotch | ✅          | ⚠️ export manual     | ✅      | ⚠️ limitado    |
| Postman    | ❌          | ❌ cloud propietario | ⚠️      | ✅ Newman      |
| Insomnia   | ⚠️          | ⚠️ export manual     | ✅      | ✅             |

**Veredicto**: Bruno para exploración manual + `axum-test` para tests
automatizados en CI. Son complementarios, no excluyentes.

### Resumen de estrategia de testing

```
Desarrollo manual    → utoipa Swagger UI  (/docs)
Tests automatizados  → axum-test          (cargo test)
Exploración/QA       → Bruno              (colecciones en repo)
CI/CD pipeline       → cargo test + bruno run --env staging
```

---

## Estructura del proyecto

```
apps/api_rust/
├── Cargo.toml
├── src/
│   ├── main.rs                  ← bootstrap: router + AppState + migraciones
│   ├── config.rs                ← env vars tipadas
│   ├── error.rs                 ← AppError → HTTP response
│   ├── auth/
│   │   ├── middleware.rs        ← extractor de token → CurrentUser
│   │   └── permissions.rs      ← workspace/project role checks
│   ├── entities/                ← generado por sea-orm-cli (NO editar a mano)
│   │   ├── issue.rs
│   │   ├── project.rs
│   │   ├── workspace.rs
│   │   ├── state.rs
│   │   └── ...
│   ├── routes/                  ← un archivo por dominio
│   │   ├── issues.rs
│   │   ├── projects.rs
│   │   ├── workspaces.rs
│   │   ├── cycles.rs
│   │   ├── modules.rs
│   │   └── integrations.rs
│   └── jobs/                    ← apalis workers + cron
│       ├── github_sync.rs
│       ├── notifications.rs
│       ├── export.rs
│       └── scheduled.rs
├── migration/                   ← sea-orm-migration crate separado
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── m20240101_000000_baseline_from_django.rs   ← dump inicial
│       └── m20240201_000001_...rs                     ← futuras migraciones
├── tests/
│   └── bruno/                   ← colecciones Bruno versionadas en git
│       ├── bruno.json
│       ├── environments/
│       └── ...
└── Dockerfile
```

---

> [!NOTE] Bruno collections
> Las colecciones Bruno viven en `apps/api_rust/tests/bruno/` y se versionan en git.
> Se pueden correr en CI con `bruno run --env staging`.

