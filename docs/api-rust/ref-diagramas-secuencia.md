---
titulo: Diagramas de secuencia
aliases:
  - diagramas-secuencia
  - sequence-diagrams
tags:
  - mermaid
  - secuencia
  - arquitectura
relacionado:
  - "[[MOC]]"
  - "[[dominio-workspace-seed]]"
  - "[[impl-extractores-auth]]"
  - "[[dominio-workspace-settings]]"
  - "[[ref-diagramas-flujo]]"
estado: activo
---

# Diagramas de secuencia

> Los diagramas de secuencia muestran el **orden temporal** de los eventos.
> Para el árbol de decisiones (if/else, guards), ver [[ref-diagramas-flujo]].

---

## 1. Creación de workspace + seed asíncrono

```mermaid
sequenceDiagram
    actor User as 🧑 Usuario
    participant Axum as Axum Handler<br/>(POST /api/workspaces/)
    participant DB as PostgreSQL
    participant Apalis as apalis<br/>(tabla apalis_jobs)
    participant Worker as WorkspaceSeed<br/>Worker (Tokio)

    User->>Axum: POST /api/workspaces/ { name, slug }
    Axum->>DB: INSERT INTO workspaces
    DB-->>Axum: workspace { id, slug }
    Axum->>DB: INSERT INTO workspace_members (owner, role=20)
    Axum->>Apalis: push(WorkspaceSeedJob { workspace_id })
    Note over Apalis: INSERT en apalis_jobs<br/>(no bloquea la respuesta)
    Axum-->>User: 201 Created { workspace }

    Note over Worker: poll cada ~1s
    Worker->>Apalis: pull job
    Worker->>DB: SELECT workspace
    Worker->>DB: INSERT bot_user
    Worker->>DB: INSERT workspace_member (bot)
    Worker->>DB: INSERT project + members + user_properties
    Worker->>DB: INSERT states × 5
    Worker->>DB: INSERT labels × 2
    Worker->>DB: INSERT cycles × 2
    Worker->>DB: INSERT modules × N
    Worker->>DB: INSERT issues × N (+ sequences + activities + label/cycle/module)
    Worker->>DB: INSERT views × N
    Worker->>DB: INSERT pages × N
    Worker-->>Apalis: job completado ✅
```

Ver detalles de implementación: [[dominio-workspace-seed]]

---

## 2. Flujo de request autenticado (issues)

```mermaid
sequenceDiagram
    actor User as 🧑 Usuario
    participant Axum as Axum Router
    participant Auth as ApiKeyUser<br/>Extractor
    participant Guard as ProjectMember<br/>Guard
    participant Repo as repositories::<br/>issues
    participant DB as PostgreSQL

    User->>Axum: GET /api/workspaces/my-ws/projects/abc/issues/
    Note over Axum: Tower middleware: tracing, CORS, gzip
    Axum->>Auth: from_request_parts()
    Auth->>DB: SELECT FROM api_tokens WHERE token = ?
    Auth->>DB: SELECT FROM users WHERE id = ?
    DB-->>Auth: Api token + user ✅
    Axum->>Guard: from_request_parts()
    Guard->>DB: SELECT workspace_members WHERE slug=? AND member_id=?
    Guard->>DB: SELECT project_members WHERE project_id=? AND member_id=?
    DB-->>Guard: ProjectMember { role } ✅
    Axum->>Repo: list_issues(db, project_id, filters)
    Repo->>DB: SELECT FROM issues WHERE deleted_at IS NULL AND project_id=?
    DB-->>Repo: Vec<issues::Model>
    Axum-->>User: 200 OK [{ id, name, state, ... }]
```

Ver extractores: [[impl-extractores-auth]]

---

## 3. Flujo OAuth — GitHub App (instalación workspace)

```mermaid
sequenceDiagram
    actor User as 🧑 Admin workspace
    participant Frontend as Frontend popup
    participant GH as GitHub App
    participant Rust as Axum Rust
    participant DB as PostgreSQL

    User->>Frontend: Click Connect GitHub
    Frontend->>GH: Abrir popup — github.com/apps/APP/installations/new?state=SLUG
    GH->>User: Pedir autorización
    User->>GH: Aprobar
    GH->>Rust: GET /api/github/callback/ installation_id=XXX state=SLUG
    Note over Rust: Sin autenticacion — GitHub redirige directamente
    Rust->>DB: SELECT workspace WHERE slug = state
    Rust->>DB: SELECT workspace_members WHERE role >= 20 admin
    Rust->>DB: UPSERT workspace_integrations SET metadata=installation_id
    Rust-->>Frontend: HTML con window.postMessage type=github-integration success=true
    Frontend->>Frontend: Popup se cierra, parent recibe el postMessage
    Frontend->>Rust: POST /api/workspaces/SLUG/workspace-integrations/github/install/
    Rust->>DB: GET OR CREATE WorkspaceIntegration
    Rust-->>Frontend: 201 workspace_integration creada
```

Ver detalles: [[dominio-integraciones#Flujo OAuth — GitHub App el más complejo]]

---

## 4. Flujo OAuth — Slack

```mermaid
sequenceDiagram
    actor User as 🧑 Admin workspace
    participant Frontend
    participant Slack as Slack OAuth
    participant Rust as Axum

    User->>Frontend: Click Connect Slack
    Frontend->>Slack: Abrir popup → slack.com/oauth/v2/authorize?client_id=...
    Slack->>User: Autorizar
    Slack-->>Frontend: Redirect con ?code=XXX
    Frontend->>Rust: POST /api/workspaces/SLUG/workspace-integrations/slack/install/ — Body: code
    Rust->>Slack: POST https://slack.com/api/oauth.v2.access — client_id + client_secret + code
    Slack-->>Rust: access_token + team_id + team_name
    Rust->>DB: UPSERT workspace_integrations metadata=slack_response
    Rust-->>Frontend: 201 workspace_integration creada
```

---

## 5. Flujo Workspace Settings — cambio de rol de miembro

```mermaid
sequenceDiagram
    actor User as 🧑 Usuario (Admin)
    participant Browser as Browser
    participant Router as React Router
    participant Layout as WorkspaceSettingLayout
    participant RBAC as RBAC (MobX store)
    participant Page as WorkspaceMembersPage
    participant API as Axum API

    User->>Browser: Navega a /{slug}/settings/members/
    Browser->>Router: match route → WorkspaceSettingLayout
    Router->>Layout: render layout
    Layout->>RBAC: getWorkspaceRoleByWorkspaceSlug(slug)
    RBAC-->>Layout: EUserWorkspaceRoles.ADMIN
    Layout->>Layout: WORKSPACE_SETTINGS_ACCESS members — ADMIN — true
    Layout->>Page: render WorkspaceMembersPage
    Page->>API: GET /api/workspaces/{slug}/members/
    API-->>Page: id + member + role + is_active
    Page->>Browser: Renderiza lista de miembros

    Note over User,Browser: Usuario cambia rol de un miembro
    User->>Page: Selecciona nuevo rol en dropdown
    Page->>API: PATCH /api/workspaces/{slug}/members/{pk}/ { role: 15 }
    API-->>Page: 200 { id, role: 15 }
    Page->>Browser: Actualiza fila del miembro en tabla
```

Ver: [[dominio-workspace-settings]]

---

## 6. Session Cookie — flujo de autenticación

```mermaid
sequenceDiagram
    participant Browser
    participant Axum as Axum SessionUser Extractor
    participant DB as PostgreSQL

    Browser->>Axum: GET /api/me/ Cookie: session-id=abc123
    Axum->>DB: SELECT FROM sessions WHERE session_key='abc123' AND expire_date > NOW()
    DB-->>Axum: session { user_id: 'uuid-del-user' }
    Axum->>DB: SELECT FROM users WHERE id = 'uuid-del-user' AND is_active = true
    DB-->>Axum: users::Model
    Axum-->>Browser: 200 OK (handler ejecutado con SessionUser)
```

Ver: [[impl-autenticacion]]

---

## 7. Entrega de webhook (evento → URL externa)

```mermaid
sequenceDiagram
    participant Handler as Axum Handler<br/>(issue/cycle/module event)
    participant Apalis as apalis<br/>(tabla apalis_jobs)
    participant Worker as WebhookDelivery<br/>Worker (Tokio)
    participant DB as PostgreSQL
    participant Target as URL externa<br/>(webhook.url)

    Handler->>DB: SELECT webhooks WHERE workspace_id=? AND is_active=true
    DB-->>Handler: Vec<webhooks::Model>
    loop Por cada webhook activo con el evento suscrito
        Handler->>Apalis: push(WebhookDeliveryJob { webhook_id, event_type, payload })
    end

    Worker->>Apalis: pull job
    Worker->>DB: SELECT webhooks WHERE id=? AND is_active=true
    DB-->>Worker: webhook { url, secret_key }
    Worker->>Worker: validate_webhook_url(url)<br/>⛔ SSRF check (RFC-1918, loopback, metadata)
    alt URL inválida / privada
        Worker-->>Apalis: job fallido — blocked SSRF (tracing::warn)
    else URL válida
        Worker->>Worker: compute_hmac_sha256(secret_key, payload)
        Worker->>Target: POST {url} — X-Plane-Signature + X-Plane-Event + JSON
        Target-->>Worker: HTTP response
        Worker->>DB: INSERT webhook_logs { status_code, response_time }
        Worker-->>Apalis: job completado ✅
    end
```

Ver detalles: [[dominio-workspace-settings#WS-6 — Webhooks]]

---

## 8. Sincronización inicial de issues — GitHub App

```mermaid
sequenceDiagram
    actor Admin as 🧑 Admin workspace
    participant API as Axum API
    participant Apalis as apalis<br/>(tabla apalis_jobs)
    participant Worker as GithubInitialIssueSync<br/>Worker (Tokio)
    participant DB as PostgreSQL
    participant GH as GitHub API

    Admin->>API: POST /workspace-integrations/github/repo-syncs/
    API->>DB: INSERT github_repository_syncs (status=queued)
    API->>Apalis: push(GithubInitialIssueSyncJob { repo_sync_id })
    API-->>Admin: 201 { repo_sync_id, status: "queued" }

    Worker->>Apalis: pull job
    Worker->>DB: SELECT repo_sync JOIN workspace_integrations
    DB-->>Worker: { installation_id, owner, repo, mappings }
    Worker->>GH: POST /app/installations/{id}/access_tokens (JWT RS256)
    GH-->>Worker: installation_token
    Worker->>DB: UPDATE github_repository_syncs SET status=started

    loop Paginación (per_page=100)
        Worker->>GH: GET /repos/{owner}/{repo}/issues?state=all&page=N
        GH-->>Worker: [ { number, title, body, state, labels } ]
        loop Por cada issue (excluir PRs)
            Worker->>DB: INSERT issues + issue_sequences + github_issue_syncs
            Worker->>DB: UPDATE imported_issues += 1
        end
    end

    Worker->>DB: UPDATE status=completed
    Worker-->>Apalis: job completado ✅
```

Ver detalles: [[dominio-integraciones#Job apalis — GithubInitialIssueSyncJob]]

---

## 9. Despacho de notificaciones (evento → in-app + email)

```mermaid
sequenceDiagram
    participant Handler as Axum Handler<br/>(issue / comment event)
    participant Apalis as apalis<br/>(tabla apalis_jobs)
    participant Worker as NotificationJob<br/>Worker (Tokio)
    participant DB as PostgreSQL
    participant EmailWorker as EmailJob<br/>(lettre / SMTP)

    Handler->>Apalis: push(NotificationJob { issue_id, actor_id, event_type })
    Note over Handler: best-effort — fallo logueado, no bloquea

    Worker->>Apalis: pull job
    Worker->>DB: get_recipients — SELECT subscribers + assignees + mencionados
    DB-->>Worker: Vec<user_id>

    loop Por cada recipient (excluir actor_id)
        Worker->>DB: check_notification_preference(user_id, event_type)
        Worker->>DB: INSERT notifications { receiver_id, title, data, read_at=NULL }
        alt should_email = true
            Worker->>DB: SELECT users WHERE id=receiver_id
            Worker->>Apalis: push(EmailJob { to, subject, html_body })
            EmailWorker->>EmailWorker: send via lettre (SMTP)
        end
    end

    Worker-->>Apalis: job completado ✅
```

Ver detalles: [[dominio-notificaciones#NotificationJob — job apalis Fase 3]]

---

## 🔗 Navegar

← [[MOC]] | → [[ref-diagramas-flujo]]

**Relacionado:** Workspace Seed: [[dominio-workspace-seed]] | Extractores: [[impl-extractores-auth]] | Integraciones: [[dominio-integraciones]] | Notificaciones: [[dominio-notificaciones]] | Settings: [[dominio-workspace-settings]]

---

_`docs/api-rust/ref-diagramas-secuencia.md` · rama `feature/integrations-panel-fix-17593507967815292912`_
