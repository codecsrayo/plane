---
titulo: Dominio — Servicios Core (Assets, Tokens, Timezones)
aliases:
  - core
  - assets
  - tokens
  - dominio-servicios-core
tags:
  - core
  - assets
  - tokens
  - dominio
  - rust
  - axum
  - pendiente-implementar
relacionado:
  - "[[MOC]]"
  - "[[dominio-issues]]"
  - "[[dominio-workspace-settings]]"
estado: activo
---

# Dominio — Servicios Core

> [!NOTE] Servicios transversales
> Gestión de archivos (assets), tokens de API para acceso programático y utilidades como zonas horarias.

---

## 1. Gestión de Archivos (Assets)

> [!WARNING] Router separado para v2 (INC-19)
> Los endpoints v2 deben vivir en un router anidado en `/assets/v2/` en Axum, separado del prefijo `/api/`.

### Endpoints v1 (Legacy)

| Método     | URL                                                               | Descripción                      |
| ---------- | ----------------------------------------------------------------- | -------------------------------- |
| `GET/POST` | `/api/workspaces/{slug}/file-assets/`                             | Subir/listar assets de workspace |
| `GET`      | `/api/workspaces/file-assets/{workspace_id}/{asset_key}/`         | Recuperar asset                  |
| `GET/POST` | `/api/users/file-assets/`                                         | Assets de usuario (e.g. avatar)  |
| `GET`      | `/api/users/file-assets/{asset_key}/`                             | Recuperar asset de usuario       |
| `POST`     | `/api/workspaces/file-assets/{workspace_id}/{asset_key}/restore/` | Restaurar asset                  |

### Endpoints v2 (Modernos)

| Método       | URL                                                                    | Descripción                 |
| ------------ | ---------------------------------------------------------------------- | --------------------------- |
| `GET/POST`   | `/assets/v2/workspaces/{slug}/`                                        | CRUD v2 assets de workspace |
| `GET/DELETE` | `/assets/v2/workspaces/{slug}/{asset_id}/`                             | Detalle / Eliminar          |
| `GET/POST`   | `/assets/v2/user-assets/`                                              | CRUD v2 assets de usuario   |
| `GET/DELETE` | `/assets/v2/user-assets/{asset_id}/`                                   | Detalle / Eliminar          |
| `POST`       | `/assets/v2/workspaces/{slug}/restore/{asset_id}/`                     | Restaurar v2                |
| `GET`        | `/assets/v2/static/{asset_id}/`                                        | Servir archivo estático     |
| `GET/POST`   | `/assets/v2/workspaces/{slug}/projects/{project_id}/`                  | Assets de proyecto          |
| `GET/DELETE` | `/assets/v2/workspaces/{slug}/projects/{project_id}/{pk}/`             | Detalle / Eliminar          |
| `POST`       | `/assets/v2/workspaces/{slug}/projects/{project_id}/{entity_id}/bulk/` | Operaciones bulk            |
| `GET`        | `/assets/v2/workspaces/{slug}/check/{asset_id}/`                       | Verificar existencia        |
| `POST`       | `/assets/v2/workspaces/{slug}/duplicate-assets/{asset_id}/`            | Duplicar asset              |
| `GET`        | `/assets/v2/workspaces/{slug}/download/{asset_id}/`                    | Descarga forzada            |

---

## 2. API Tokens

**Propósito:** Permitir acceso programático a la API mediante tokens de larga duración.

| Método       | URL                           | Guard         |
| ------------ | ----------------------------- | ------------- |
| `GET/POST`   | `/api/users/api-tokens/`      | `CurrentUser` |
| `GET/DELETE` | `/api/users/api-tokens/{pk}/` | `CurrentUser` |

> [!COMMENT] Nota de implementación en Rust
> Los API Tokens se validan en un middleware de autenticación que busca el header `X-API-Key` o `Authorization: Api-Key <token>`.

---

## 3. Timezones

**Propósito:** Proveer lista de zonas horarias válidas para la configuración del usuario.

| Método | URL               | Descripción                             |
| ------ | ----------------- | --------------------------------------- |
| `GET`  | `/api/timezones/` | Retorna lista de strings IANA timezones |

---

## Entidades SeaORM involucradas ✅

| Entidad          | Tabla         |
| ---------------- | ------------- |
| `file_assets.rs` | `file_assets` |
| `api_tokens.rs`  | `api_tokens`  |

---

## Plan de implementación

```
Fase 2:
  [ ] src/routes/assets.rs         — GET/DELETE /assets/v2/{asset_id}/
                                     POST /assets/v2/entity/ (presigned upload)
  [ ] src/routes/api_tokens.rs     — CRUD /api-tokens/
  [ ] src/routes/timezones.rs      — GET /timezones/ (lista estática)
  [ ] src/utils/s3.rs              — generate_presigned_url + delete_asset (aws-sdk-s3)
```

## 🔗 Navegar

← [[dominio-usuario]] | [[MOC]] | → [[dominio-busqueda]]
