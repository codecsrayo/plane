# Auditoría de Sincronización: Rust API DTOs vs TypeScript Interfaces

Esta auditoría detalla las discrepancias encontradas entre los objetos de transferencia de datos (DTOs) retornados por la API de Rust y las interfaces de TypeScript definidas en el frontend (`packages/types/src/`).

## Resumen de Hallazgos Generales

- **Nomenclatura de IDs**: Rust tiende a usar sufijos `_id` internamente, pero muchos DTOs ya aplican `#[serde(rename = "...")]` para coincidir con la convención de Django/Frontend (ej. `parent_id` -> `parent`).
- **Tipos de Datos**: Rust retorna UUIDs y fechas en formato ISO, mientras que el frontend los trata mayormente como `string` o `Date`.
- **Anidamiento**: En varios casos, el frontend espera objetos anidados (ej. `owner: IUser`), pero la API de Rust retorna solo el ID (`owner_id: Uuid`).
- **Campos de Auditoría**: Rust incluye campos como `created_at`, `updated_at`, `created_by` y `updated_by` en casi todos los DTOs, los cuales a veces faltan en las interfaces de TS.

---

## 1. Workspaces

**Rust DTO**: `WorkspaceResponse` | **TS Interface**: `IWorkspace`

| Campo              | Rust Type        | TS Type        | Novedad / Discrepancia                                                    |
| :----------------- | :--------------- | :------------- | :------------------------------------------------------------------------ |
| `owner`            | `owner_id: Uuid` | `owner: IUser` | Rust retorna solo el ID; TS espera el objeto de usuario completo.         |
| `logo`             | `Option<String>` | --             | Rust retorna el campo crudo `logo` (URL o path), TS no lo tiene definido. |
| `background_color` | `String`         | --             | Rust lo incluye, TS no lo tiene definido.                                 |
| `url`              | --               | `string`       | TS espera `url`, Rust no la retorna.                                      |
| `total_projects`   | --               | `number`       | TS espera `total_projects`, Rust no lo incluye en el listado base.        |

## 2. Projects

**Rust DTO**: `ProjectResponse` / `ProjectListResponse` | **TS Interface**: `IProject` / `IPartialProject`

| Campo              | Rust Type                   | TS Type                       | Novedad / Discrepancia                                       |
| :----------------- | :-------------------------- | :---------------------------- | :----------------------------------------------------------- |
| `workspace`        | `workspace_id: Uuid`        | `IWorkspace \| string`        | Rust usa `rename`, pero TS puede recibir el objeto completo. |
| `default_assignee` | `default_assignee_id: Uuid` | `IUser \| string \| null`     | Rust retorna solo ID, TS permite objeto.                     |
| `project_lead`     | `project_lead_id: Uuid`     | `IUserLite \| string \| null` | Rust retorna solo ID, TS permite objeto.                     |
| `default_state`    | `default_state_id: Uuid`    | `string \| null`              | Nombres de campos no coinciden (`_id` vs sin sufijo).        |
| `estimate`         | `estimate_id: Uuid`         | `string \| null`              | Nombres de campos no coinciden (`_id` vs sin sufijo).        |
| `anchor`           | `Option<String>`            | `string \| null`              | Solo presente en `ProjectDetailResponse` en Rust.            |

## 3. Issues (Work Items)

**Rust DTO**: `ProjectIssueItem` | **TS Interface**: `TBaseIssue`

| Campo            | Rust Type          | TS Type          | Novedad / Discrepancia                                                            |
| :--------------- | :----------------- | :--------------- | :-------------------------------------------------------------------------------- |
| `estimate_point` | `Option<Uuid>`     | `string \| null` | Rust usa `estimate_point` (sin `_id`) para paridad con Django.                    |
| `state__group`   | `Option<String>`   | --               | Presente en Rust (paridad Django), pero falta en `TBaseIssue` (está en `TIssue`). |
| `is_epic`        | --                 | `boolean`        | TS tiene `is_epic`, Rust no lo incluye en este DTO.                               |
| `is_intake`      | `bool` (en Detail) | `boolean`        | Rust solo lo incluye en `IssueDetailResponse`.                                    |

## 4. States

**Rust DTO**: `StateResponse` | **TS Interface**: `IState`

| Campo                       | Rust Type       | TS Type  | Novedad / Discrepancia                                                  |
| :-------------------------- | :-------------- | :------- | :---------------------------------------------------------------------- |
| `order`                     | `f64`           | `number` | Rust incluye un alias `order` del campo `sequence` para compatibilidad. |
| `slug`                      | `String`        | --       | Rust retorna `slug`, TS no lo tiene.                                    |
| `is_triage`                 | `bool`          | --       | Rust retorna `is_triage`, TS no lo tiene.                               |
| `created_at` / `updated_at` | `DateTime<Utc>` | --       | Campos de auditoría faltantes en TS.                                    |

## 5. Cycles

**Rust DTO**: `CycleResponse` | **TS Interface**: `ICycle`

| Campo            | Rust Type | TS Type           | Novedad / Discrepancia                          |
| :--------------- | :-------- | :---------------- | :---------------------------------------------- |
| `project_detail` | --        | `IProjectDetails` | TS espera `project_detail`, Rust no lo incluye. |
| `progress`       | --        | `any[]`           | TS espera `progress`, Rust no lo incluye.       |
| `sub_issues`     | --        | `number`          | TS tiene `sub_issues`, Rust usa `total_issues`. |

## 6. Modules

**Rust DTO**: `ModuleResponse` | **TS Interface**: `IModule`

| Campo              | Rust Type | TS Type          | Novedad / Discrepancia                                              |
| :----------------- | :-------- | :--------------- | :------------------------------------------------------------------ |
| `description_text` | --        | `any`            | TS espera `description_text`, Rust solo tiene `description` (HTML). |
| `description_html` | --        | `any`            | TS espera `description_html`, Rust solo tiene `description`.        |
| `link_module`      | --        | `ILinkDetails[]` | TS espera `link_module`, Rust no lo incluye en el DTO base.         |
| `sub_issues`       | --        | `number`         | TS tiene `sub_issues`, Rust usa `total_issues`.                     |

## 7. Pages

**Rust DTO**: `PageResponse` | **TS Interface**: `TPage`

| Campo        | Rust Type            | TS Type  | Novedad / Discrepancia              |
| :----------- | :------------------- | :------- | :---------------------------------- |
| `owned_by`   | `owned_by_id: Uuid`  | `string` | Rust aplica `rename` correctamente. |
| `workspace`  | `workspace_id: Uuid` | `string` | Rust aplica `rename` correctamente. |
| `deleted_at` | `Option<DateTime>`   | `Date`   | Tipos compatibles.                  |

## 8. Users

**Rust DTO**: `UserMeResponse` | **TS Interface**: `IUser`

| Campo          | Rust Type | TS Type      | Novedad / Discrepancia                                                                 |
| :------------- | :-------- | :----------- | :------------------------------------------------------------------------------------- |
| `is_superuser` | `bool`    | --           | Rust lo incluye, TS no lo tiene.                                                       |
| `is_managed`   | `bool`    | --           | Rust lo incluye, TS no lo tiene.                                                       |
| `theme`        | --        | `IUserTheme` | TS incluye `theme` en el objeto usuario, Rust lo retorna en el DTO de Perfil separado. |
