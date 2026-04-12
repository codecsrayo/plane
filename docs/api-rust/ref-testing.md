---
titulo: Estrategia de testing
aliases:
  - testing
  - bruno
  - axum-test
  - swagger-ui
tags:
  - testing
  - bruno
  - axum-test
  - swagger
relacionado:
  - "[[MOC]]"
  - "[[impl-bootstrap]]"
  - "[[ref-estructura-archivos]]"
  - "[[impl-extractores-auth]]"
estado: activo
---

# Estrategia de testing

---

## Resumen de estrategia

```
Desarrollo manual    → utoipa Swagger UI     (/api/docs)
Tests automatizados  → axum-test             (cargo test)
Exploración/QA       → Bruno                 (colecciones en repo)
CI/CD pipeline       → cargo test + bruno run --env staging
```

---

## 1. Swagger UI — utoipa

`utoipa` genera el spec OpenAPI 3.x a partir de macros Rust. Se monta en Axum:

```toml
# Cargo.toml
utoipa = { version = "5.4.0", features = ["axum_extras", "uuid", "chrono"] }
utoipa-swagger-ui = { version = "9.0.2", features = ["axum"] }
```

```rust
// src/routes/mod.rs — montar Swagger UI en /api/docs
use utoipa_swagger_ui::SwaggerUi;

let swagger = SwaggerUi::new("/api/docs")
    .url("/api/docs/openapi.json", ApiDoc::openapi());

Router::new()
    .nest("/api", api_router)
    .merge(swagger)
    .with_state(state)
```

Acceder a `http://localhost:8000/api/docs` para explorar y probar endpoints manualmente.

### Autorización en Swagger UI

1. Abrir `http://localhost:8000/api/docs`
2. Click en "Authorize" (candado 🔒)
3. En `TokenAuth (apiKey)` ingresar: `Token <tu_token_aquí>`
4. Click "Authorize" y cerrar
5. Todos los requests subsiguientes llevarán el header `Authorization: Token ...`

---

## 2. Tests automatizados — `axum-test`

Permite hacer requests HTTP directamente al router de Axum sin levantar un servidor TCP real. Ideal para tests de integración rápidos:

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

### Tests de extractores de auth

```rust
// tests/auth_extractors.rs
#[tokio::test]
async fn test_no_token_returns_401() {
    let server = build_test_app().await;
    let resp = server.get("/api/workspaces/my-ws/projects/abc/issues").await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_non_member_returns_403() {
    let server = build_test_app().await;
    let resp = server
        .get("/api/workspaces/other-ws/projects/abc/issues")
        .add_header("Authorization", "Token valid_token_other_user")
        .await;
    resp.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_member_can_list_issues() {
    let server = build_test_app().await;
    let resp = server
        .get("/api/workspaces/my-ws/projects/abc/issues")
        .add_header("Authorization", "Token valid_token_member")
        .await;
    resp.assert_status(StatusCode::OK);
}
```

### Alternativa ligera — `httpc-test`

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

---

## 3. Bruno — cliente externo

Bruno es open source, almacena las colecciones como archivos en el repo (no en la nube), y funciona offline. Las colecciones viven en `apps/api_rust/tests/bruno/`.

```bash
# Instalar Bruno
brew install bruno  # macOS
# o descargar desde https://www.usebruno.com/

# Correr colección en CI
bruno run --env staging tests/bruno/
```

### Estructura de la colección

```tree
apps/api_rust/tests/bruno/
├── bruno.json
├── environments/
│   ├── local.bru
│   └── staging.bru
├── health/
│   └── get_health.bru
├── workspaces/
│   ├── list_workspaces.bru
│   ├── create_workspace.bru
│   └── get_workspace.bru
├── issues/
│   ├── list_issues.bru
│   ├── create_issue.bru
│   └── update_issue.bru
└── integrations/
    ├── list_integrations.bru
    └── connect_github.bru
```

### Comparativa de clientes externos

| Cliente    | Open Source |   Archivos en git    | Offline |     CI/CD      |
| ---------- | :---------: | :------------------: | :-----: | :------------: |
| **Bruno**  |     ✅      |  ✅ archivos `.bru`  |   ✅    | ✅ `bruno run` |
| Hoppscotch |     ✅      |   ⚠️ export manual   |   ✅    |  ⚠️ limitado   |
| Postman    |     ❌      | ❌ cloud propietario |   ⚠️    |   ✅ Newman    |
| Insomnia   |     ⚠️      |   ⚠️ export manual   |   ✅    |       ✅       |

**Veredicto:** Bruno para exploración manual + `axum-test` para tests automatizados en CI. Son complementarios.

---

## 4. Pipeline CI recomendado

```yaml
# .github/workflows/rust.yml
name: Rust API Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_DB: plane_test
          POSTGRES_USER: plane
          POSTGRES_PASSWORD: plane
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v4
      - name: Apply migrations
        run: cargo run -p migration -- up
        env:
          DATABASE_URL: postgresql://plane:plane@localhost:5432/plane_test

      - name: Run tests
        run: cargo test --all
        env:
          DATABASE_URL: postgresql://plane:plane@localhost:5432/plane_test

      - name: Run Bruno smoke tests
        run: bruno run --env staging tests/bruno/health/
```

---

## 🔗 Navegar

← [[impl-bootstrap]] | [[MOC]] | Estructura: [[ref-estructura-archivos]] | Extractores: [[impl-extractores-auth]]
