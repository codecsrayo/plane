---
titulo: Dominio — Usuario (Perfil y Preferencias)
aliases:
  - user
  - usuario
  - perfil
  - dominio-usuario
tags:
  - user
  - profile
  - dominio
  - rust
  - axum
relacionado:
  - "[[MOC]]"
  - "[[impl-autenticacion]]"
  - "[[dominio-workspace-settings]]"
estado: activo
---

# Dominio — Usuario (Perfil y Preferencias)

> [!NOTE] INC-18 — Dominio de usuario
> Este dominio no estaba documentado previamente.

> [!NOTE] Gestión del usuario autenticado
> Endpoints bajo `/users/me/` para el perfil, configuración, cuentas vinculadas y dashboards personales.

---

## Modelo de datos

```
users (tabla principal)
    ├─ profiles (perfil extendido)
    ├─ accounts (cuentas OAuth vinculadas: github, google, etc.)
    ├─ sessions (dispositivos y sesiones activas)
    └─ user_notification_preferences (configuración de correo/in-app)
```

---

## Endpoints a implementar

> [!INFO] Fuente
> `apps/api/plane/app/urls/user.py` — Django.

### Perfil y cuenta — `UserEndpoint`

| Método   | URL                              | Descripción                              | Fase |
| -------- | -------------------------------- | ---------------------------------------- | ---- |
| `GET`    | `/users/me/`                     | Datos del usuario actual                 | 1    |
| `PATCH`  | `/users/me/`                     | Actualizar perfil (nombre, avatar)       | 4    |
| `DELETE` | `/users/me/`                     | Desactivar cuenta (soft delete)          | 5    |
| `GET`    | `/users/me/settings/`            | Configuración del usuario                | 4    |
| `POST`   | `/users/me/email/generate-code/` | Generar código de verificación de correo | 4    |
| `PATCH`  | `/users/me/email/`               | Cambiar correo electrónico               | 4    |
| `GET`    | `/users/me/instance-admin/`      | ¿Es administrador de la instancia?       | 4    |
| `GET`    | `/users/session/`                | Info de sesión actual                    | 1    |

### Perfil extendido y Onboarding

| Método      | URL                         | Descripción                                          | Fase |
| ----------- | --------------------------- | ---------------------------------------------------- | ---- |
| `GET/PATCH` | `/users/me/profile/`        | `ProfileEndpoint` — rol, compañía, onboarding status | 1    |
| `POST`      | `/users/me/onboard/`        | Marcar onboarding como completado                    | 1    |
| `POST`      | `/users/me/tour-completed/` | Marcar tour inicial como completado                  | 4    |

### Cuentas OAuth — `AccountEndpoint`

| Método   | URL                        | Descripción                                     | Fase |
| -------- | -------------------------- | ----------------------------------------------- | ---- |
| `GET`    | `/users/me/accounts/`      | Lista cuentas vinculadas (GitHub, Google, etc.) | 4    |
| `DELETE` | `/users/me/accounts/{pk}/` | Desvincular cuenta                              | 4    |

### Actividad y Dashboards

| Método | URL                                                   | Descripción                                  | Fase |
| ------ | ----------------------------------------------------- | -------------------------------------------- | ---- |
| `GET`  | `/users/me/workspaces/`                               | Workspaces a los que pertenece el usuario    | 2    |
| `GET`  | `/users/me/activities/`                               | Historial de actividad del usuario           | 4    |
| `GET`  | `/users/me/workspaces/{slug}/activity-graph/`         | Datos para el heatmap de actividad           | 4    |
| `GET`  | `/users/me/workspaces/{slug}/issues-completed-graph/` | Histograma de issues completados             | 4    |
| `GET`  | `/users/me/workspaces/{slug}/dashboard/`              | Resumen del dashboard personal del workspace | 4    |

---

## Handler — `GET /users/me/`

Endpoint fundamental para el arranque del frontend.

```rust
pub async fn get_me(
    CurrentUser(user): CurrentUser,
    State(state): State<AppState>,
) -> Result<Json<UserResponse>, AppError> {
    let db = &state.db;

    // Obtener perfil relacionado
    let profile = profiles::Entity::find()
        .filter(profiles::Column::UserId.eq(user.id))
        .one(db).await.map_err(AppError::Database)?;

    Ok(Json(UserResponse::from_model(user, profile)))
}
```

---

## DTOs

```rust
#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub id:         Uuid,
    pub email:      String,
    pub first_name: String,
    pub last_name:  String,
    pub avatar:     Option<String>,
    pub is_active:  bool,
    pub is_bot:     bool,
    pub onboarding_step: Option<serde_json::Value>,
    // Perfil
    pub role:       Option<String>,
    pub company:    Option<String>,
    pub use_case:   Option<String>,
}
```

---

## Entidades SeaORM involucradas ✅

| Entidad                 | Tabla                |
| ----------------------- | -------------------- |
| `users.rs`              | `users`              |
| `profiles.rs`           | `profiles`           |
| `accounts.rs`           | `accounts`           |
| `user_recent_visits.rs` | `user_recent_visits` |

---

## 🔗 Navegar

← [[dominio-vistas]] | [[MOC]] | → [[impl-autenticacion]]
