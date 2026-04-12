---
titulo: Diagramas de flujo — Workspace y Projects
aliases:
  - diagramas-flujo
  - flowcharts
tags:
  - mermaid
  - flowchart
  - workspace
  - projects
  - arquitectura
relacionado:
  - "[[MOC]]"
  - "[[impl-extractores-auth]]"
  - "[[ref-diagramas-secuencia]]"
  - "[[dominio-workspace-settings]]"
  - "[[plan-fases]]"
estado: activo
---

# Diagramas de flujo — Workspace y Projects

> Los flowcharts muestran el **árbol de decisiones** (if/else, guards).
> Para el orden temporal de los eventos, ver [[ref-diagramas-secuencia]].

---

## Workspace — flujo de creación y acceso

```mermaid
flowchart TD
    A([🧑 Request llega al router Axum]) --> B{¿Token Bearer\npresente?}

    B -- No --> ERR401[401 Unauthorized]
    B -- Sí --> C[CurrentUser extractor]

    C --> D{¿Token válido\ny usuario activo?}
    D -- No --> ERR401
    D -- Sí --> E{¿Tipo de operación?}

    E -- Crear workspace --> CREATE[POST /api/workspaces/]
    E -- Operar en workspace --> F[WorkspaceMemberGuard]

    CREATE --> G[repositories::workspaces::create]
    G --> H[(INSERT workspaces)]
    H --> I[(INSERT workspace_members role=20)]
    I --> J[apalis: push WorkspaceSeedJob]
    J --> K([201 Created])

    F --> L{¿Es miembro?}
    L -- No --> ERR403[403 Forbidden]
    L -- Sí --> M{¿Rol requerido?}

    M -- "Lectura (role ≥ 5)" --> READ_WS[GET /workspaces/:slug/]
    M -- "Admin (role ≥ 20)" --> ADMIN_WS[PATCH / DELETE /workspaces/:slug/]

    READ_WS --> N[repositories::workspaces::get_by_slug]
    ADMIN_WS --> O{¿DELETE?}
    O -- Sí --> P{¿Es Owner?}
    P -- No --> ERR403
    P -- Sí --> Q[UPDATE deleted_at = NOW]
    O -- No --> R[UPDATE workspaces]

    N --> RESP200([200 OK])
    R --> RESP200
    Q --> RESP204([204 No Content])
```

---

## Workspace Seed Worker — flujo interno

```mermaid
flowchart TD
    START([apalis poll: WorkspaceSeedJob]) --> WS[SELECT workspace]
    WS --> BOT[INSERT users bot_user]
    BOT --> WM[INSERT workspace_members bot]
    WM --> PROJ[INSERT projects 'Default Project']
    PROJ --> PM[INSERT project_members owner + bot]
    PM --> UP[INSERT user_properties × miembros]
    UP --> ST[INSERT states × 5]
    ST --> LB[INSERT labels × 2]
    LB --> CY[INSERT cycles × 2]
    CY --> MOD[INSERT modules × N]
    MOD --> IS[INSERT issues × N\n+ sequences + activities\n+ label_issue / cycle_issue / module_issue]
    IS --> VW[INSERT views × N]
    VW --> PG[INSERT pages × N]
    PG --> DONE([✅ job completado])
```

---

## Projects — guard chain y acceso

```mermaid
flowchart TD
    REQ([Request a /workspaces/:slug/projects/:id/**]) --> CU[CurrentUser extractor]
    CU --> F1{¿Token OK?}
    F1 -- No --> E401[401 Unauthorized]
    F1 -- Sí --> WG[WorkspaceMemberGuard]

    WG --> F2{¿Es miembro\ndel workspace?}
    F2 -- No --> E403[403 Forbidden]
    F2 -- Sí --> PG[ProjectMemberGuard]

    PG --> F3{¿Es miembro\ndel proyecto?}
    F3 -- No --> E403
    F3 -- Sí --> F4{¿Rol requerido?}

    F4 -- "Lectura (role ≥ 5)" --> ROPS[GET issues / states\n/ members / cycles]
    F4 -- "Escritura (role ≥ 15)" --> WOPS[POST / PATCH issues]
    F4 -- "Admin (role ≥ 18)" --> AOPS[DELETE proyecto\ngestión miembros]

    ROPS --> REPO[repositories::\nfiltro: deleted_at IS NULL]
    WOPS --> REPO
    AOPS --> F5{¿Role OK?}
    F5 -- No --> E403
    F5 -- Sí --> REPO

    REPO --> DB[(PostgreSQL)]
    DB --> RESP([200 / 201 / 204])
```

---

## Issues — CRUD completo

```mermaid
flowchart TD
    REQ([Request a /projects/:id/issues/**]) --> GRD[ProjectMemberGuard]
    GRD --> OP{¿Operación?}

    OP -- "GET /issues/" --> LIST[list_issues\ncon IssueFilters]
    OP -- "POST /issues/" --> CREATE[create_issue]
    OP -- "GET /issues/:id/" --> DETAIL[get_issue_by_id]
    OP -- "PATCH /issues/:id/" --> UPDATE[update_issue\nrole ≥ 15]
    OP -- "DELETE /issues/:id/" --> DELETE[soft_delete\nrole ≥ 15]

    LIST --> Q1[(SELECT issues WHERE\nproject_id=? AND\ndeleted_at IS NULL\nfiltros dinámicos)]

    CREATE --> VAL{¿Estado válido\ndentro del proyecto?}
    VAL -- No --> E422[422 Unprocessable Entity]
    VAL -- Sí --> INS[(INSERT issues\n+ IssueSequence\n+ IssueActivity)]

    Q1 --> RESP200([200 OK — Vec issues])
    INS --> RESP201([201 Created — issue])
    DETAIL --> Q3[(SELECT WHERE id=? AND deleted_at IS NULL)]
    Q3 --> RESP200
    UPDATE --> UPD[(UPDATE issues + INSERT IssueActivity)]
    UPD --> RESP200
    DELETE --> SD[(UPDATE deleted_at = NOW)]
    SD --> RESP204([204 No Content])
```

---

## Roles — jerarquía por entidad

```mermaid
flowchart LR
    subgraph WS_ROLES["Workspace roles"]
        WR1["Guest (5)\nSolo lectura básica"]
        WR2["Viewer (10)\nLectura de todo"]
        WR3["Member (15)\nCrear proyectos, issues"]
        WR4["Admin / Owner (20)\nGestión completa"]
        WR1 --> WR2 --> WR3 --> WR4
    end

    subgraph PJ_ROLES["Project roles"]
        PR1["Guest (5)"]
        PR2["Viewer (10)"]
        PR3["Member (15)\nCRUD issues, cycles"]
        PR4["Admin (18)\nGestión proyecto"]
        PR1 --> PR2 --> PR3 --> PR4
    end

    subgraph GUARDS["Guards en Axum"]
        GU1["CurrentUser\n→ 401 si token inválido"]
        GU2["WorkspaceMemberGuard\n→ 403 si no es miembro WS"]
        GU3["ProjectMemberGuard\n→ 403 si no es miembro proyecto"]
        GU1 --> GU2 --> GU3
    end

    WS_ROLES -- "verifica" --> GU2
    PJ_ROLES -- "verifica" --> GU3
```

---

## Relación entidades Workspace ↔ Projects

```mermaid
flowchart TD
    WS[(workspaces)] --> WM[(workspace_members\nrole: 5/10/15/20)]
    WS --> WI[(workspace_member_invites)]
    WS --> WK[(workspace_integrations\ngithub / gitlab / slack)]
    WS --> WH[(webhooks)]
    WS --> WE[(exporter_histories)]

    WS --> PJ[(projects\nidentifier único)]
    PJ --> PM[(project_members\nrole: 5/10/15/18)]
    PJ --> ST[(states × 5)]
    PJ --> LB[(labels)]
    PJ --> IS[(issues)]
    PJ --> CY[(cycles)]
    PJ --> MOD[(modules)]
    PJ --> VW[(views)]
    PJ --> PG[(pages)]

    IS --> IA[(issue_activities)]
    IS --> IC[(issue_comments)]
    IS --> IL[(label_issue)]
    IS --> ICI[(cycle_issue)]
    IS --> IMI[(module_issue)]
```

---

## Tabla de rutas por entidad

| Entidad        | Método           | Ruta                                              | Guard mínimo                            | Fase |
| -------------- | ---------------- | ------------------------------------------------- | --------------------------------------- | ---- |
| Workspaces     | GET              | `/api/workspaces/`                                | `CurrentUser`                           | 2    |
| Workspace      | POST             | `/api/workspaces/`                                | `CurrentUser`                           | 2    |
| Workspace      | GET/PATCH/DELETE | `/api/workspaces/:slug/`                          | `WorkspaceMemberGuard`                  | 2    |
| WS Members     | GET              | `/api/workspaces/:slug/members/`                  | `WorkspaceMemberGuard (≥15)`            | 2    |
| WS Members     | PATCH/DELETE     | `/api/workspaces/:slug/members/:pk/`              | `WorkspaceMemberGuard (≥20)`            | 2    |
| WS Invitations | GET/POST         | `/api/workspaces/:slug/invitations/`              | `WorkspaceMemberGuard (≥20)`            | 2    |
| WS Webhooks    | GET/POST         | `/api/workspaces/:slug/webhooks/`                 | `WorkspaceMemberGuard (≥20)`            | 2    |
| Projects       | GET              | `/api/workspaces/:slug/projects/`                 | `WorkspaceMemberGuard (≥5)`             | 2    |
| Project        | POST             | `/api/workspaces/:slug/projects/`                 | `WorkspaceMemberGuard (≥15)`            | 2    |
| Project        | GET/PATCH/DELETE | `/api/workspaces/:slug/projects/:id/`             | `ProjectMemberGuard (≥18 PATCH/DELETE)` | 2    |
| Issues         | GET/POST         | `/api/workspaces/:slug/projects/:id/issues/`      | `ProjectMemberGuard (≥5 GET, ≥15 POST)` | 2    |
| Issue          | GET/PATCH/DELETE | `/api/workspaces/:slug/projects/:id/issues/:iid/` | `ProjectMemberGuard (≥15 PATCH/DELETE)` | 2    |
| States         | GET              | `/api/workspaces/:slug/projects/:id/states/`      | `ProjectMemberGuard (≥5)`               | 2    |
| Members        | GET              | `/api/workspaces/:slug/projects/:id/members/`     | `ProjectMemberGuard (≥5)`               | 2    |
| Cycles         | GET/POST         | `/api/workspaces/:slug/projects/:id/cycles/`      | `ProjectMemberGuard (≥5 GET, ≥15 POST)` | 2    |
| Modules        | GET/POST         | `/api/workspaces/:slug/projects/:id/modules/`     | `ProjectMemberGuard (≥5 GET, ≥15 POST)` | 2    |
| Exports        | GET/POST         | `/api/workspaces/:slug/exports/`                  | `WorkspaceMemberGuard (≥5)`             | 3    |

---

## Extractor chain — `ProjectMemberGuard` internamente

Muestra las 5 queries secuenciales que dispara `ProjectMemberGuard` antes de que llegue al handler.

```mermaid
flowchart TD
    REQ([HTTP Request]) --> AX[Axum Router\nmatch route]
    AX --> E1[Extractor 1: State\nAppState]
    E1 --> E2[Extractor 2: ProjectMemberGuard]

    subgraph PMG[ProjectMemberGuard internamente]
        direction TB
        T1[Leer header Authorization] --> T2{Formato Token XYZ?}
        T2 -- No --> R401A[Reject 401]
        T2 -- Sí --> T3[SELECT authtoken_token WHERE key=?]
        T3 --> T4{Existe?}
        T4 -- No --> R401B[Reject 401]
        T4 -- Sí --> T5[SELECT users WHERE id=?]
        T5 --> T6{is_active?}
        T6 -- No --> R401C[Reject 401]
        T6 -- Sí --> T7[Leer :slug del path]
        T7 --> T8[SELECT workspaces WHERE slug=?]
        T8 --> T9{Existe?}
        T9 -- No --> R404A[Reject 404]
        T9 -- Sí --> T10[SELECT workspace_members]
        T10 --> T11{Es miembro WS?}
        T11 -- No --> R403A[Reject 403]
        T11 -- Sí --> T12[Leer :project_id]
        T12 --> T13[SELECT projects WHERE id=?]
        T13 --> T14{Existe?}
        T14 -- No --> R404B[Reject 404]
        T14 -- Sí --> T15[SELECT project_members]
        T15 --> T16{Es miembro proyecto?}
        T16 -- No --> R403B[Reject 403]
        T16 -- Sí --> OK[Ok — ProjectMemberGuard]
    end

    R401A & R401B & R401C --> RESP401([Response 401])
    R403A & R403B --> RESP403([Response 403])
    R404A & R404B --> RESP404([Response 404])
    OK --> E3[Extractor 3: Path params]
    E3 --> E4[Extractor 4: Json body]
    E4 --> HANDLER[Handler]
    HANDLER --> RESP200([Response 200/201/204])
```

---

## 🔗 Navegar

← [[ref-diagramas-secuencia]] | [[MOC]]

**Relacionado:** Extractores: [[impl-extractores-auth]] | Workspace Settings: [[dominio-workspace-settings]] | Fases: [[plan-fases]]
