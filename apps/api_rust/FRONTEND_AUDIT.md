# Frontend Audit — API Requests vs api_rust

**Generated:** 2026-05-24  
**Scope:** `apps/web`, `apps/space`, `apps/admin`, `packages/services`  
**Objective:** Confirm 100% of frontend API calls target the Rust backend; identify any gaps.

---

## 1. Configuration Audit

### API Base URL

| Setting | Value | Verdict |
|---------|-------|---------|
| `packages/constants/src/endpoints.ts` | `process.env.VITE_API_BASE_URL \|\| ""` | ✅ Uses env var, no hardcode |
| `apps/web/.env.example` `VITE_API_BASE_URL` | `http://localhost:8000` | ✅ Rust API port |
| `docker-compose.yml` `api` service | `apps/api_rust/Dockerfile`, port 8000 | ✅ Django removed |
| `docker-compose-dev.yml` `api` service | `apps/api_rust/Dockerfile.dev`, port 8000 | ✅ Django removed |

All three frontend apps (`web`, `space`, `admin`) and all `packages/services` service classes
inherit `API_BASE_URL` → `VITE_API_BASE_URL` → Rust API. **No service bypasses this base URL.**

### Hardcoded URL scan

| Location | Pattern | Verdict |
|----------|---------|---------|
| `packages/e2e-utils/` | `process.env.API_BASE_URL ?? "http://localhost:8000"` | ✅ E2E test utils only; not production code |
| `apps/space/app/…/layout.tsx` | `${process.env.VITE_API_BASE_URL}/api/public/anchor/…/meta/` | ✅ Correct env var usage |
| All service classes | No raw `http://` URLs | ✅ |

---

## 2. Endpoints Validated (FE → Rust ✅)

The following areas were audited and all endpoints are implemented in Rust.
See `todo_frontend.md` for the detailed per-endpoint breakdown with fix notes.

| Area | Status |
|------|--------|
| Auth (sign-in/up, magic, OAuth, password, God Mode) | ✅ All implemented |
| Health / Timezones | ✅ |
| Users (me, settings, profile, accounts, email, workspaces, dashboard) | ✅ |
| Workspaces (CRUD, members, themes, invitations) | ✅ |
| Workspace extras (favorites, stickies, sidebar, home-prefs, quick-links, recent-visits, draft-issues) | ✅ |
| Projects (CRUD, members, invitations, archive, identifiers, deploy-boards, user-properties) | ✅ |
| States, Labels, Estimates | ✅ |
| Issues (CRUD, archive, bulk-delete, bulk-archive, bulk-operation, issue-dates, meta, identifier-lookup) | ✅ |
| Issue extras (comments, reactions, links, relations, subscribers, activities, versions) | ✅ |
| Issue attachments (v2 + legacy) | ✅ |
| Cycles (CRUD, issues, analytics, progress, archive, favorites, transfer, date-check) | ✅ |
| Modules (CRUD, issues, links, archive, favorites) | ✅ |
| Pages (CRUD, archive, lock, duplicate, move, versions, description, favorites) | ✅ |
| Intake/Inbox (CRUD) | ✅ |
| Views (workspace + project) | ✅ |
| Notifications | ✅ |
| Webhooks | ✅ |
| Search (global, work-items, project-level, entity-search) | ✅ |
| Analytics (workspace, advance, project) | ✅ |
| Assets v2 (user, workspace, project, bulk) | ✅ |
| Legacy file assets cleanup | ✅ |
| Exporter (export-issues) | ✅ |
| Importers (GitHub, GitLab, list-all) | ✅ |
| Integrations (GitHub, GitLab, workspace-integrations) | ✅ |
| API Tokens | ✅ |
| Instance (CRUD, admins, configurations, workspace-slug-check) | ✅ |
| API v1 public (work-items, issues, cycles, modules, assets) | ✅ |

---

## 3. Gaps Found

### 3a. Frontend Calling Wrong Path (FE Bug — No Rust Fix Needed)

These are paths the frontend calls that **don't exist** in Django CE or Rust.
The correct paths ARE implemented in Rust.

| FE Calls | Correct Path (Rust has it) | Notes |
|----------|---------------------------|-------|
| `GET/POST …/projects/{id}/issue-display-properties/` | `…/projects/{id}/user-properties/` | Django URL *name* is "project-issue-display-properties" but the *path* is `user-properties/`. FE has wrong path. See `issue.service.ts:173,185`. |
| `GET …/cycles/{id}/cycle-progress/` | `…/cycles/{id}/progress/` | Both Django CE and Rust use `/progress/`. FE calls `/cycle-progress/`. See `cycle.service.ts:57`. |

**Action:** These are FE bugs. Both correct endpoints exist in Rust. No Rust changes required.

### 3b. EE-Only Features (Not in Django CE — Deferred)

These endpoints are not present in Django CE. They appear to be Enterprise Edition features.
The Space app and web app may degrade gracefully when they 404.

| Endpoint | Called By | Django CE? | Notes |
|----------|-----------|-----------|-------|
| `GET /api/workspaces/{slug}/my-issues/` | `UserService.userIssues` | ❌ EE only | Returns 404 silently |
| `GET /api/workspaces/{slug}/dashboard/` | `DashboardService.getHomeDashboardWidgets` | ❌ EE only | CE uses `/users/me/workspaces/{slug}/dashboard` instead |
| `GET /api/workspaces/{slug}/dashboard/{id}/` | `DashboardService.getWidgetStats` | ❌ EE only | |
| `GET/PATCH /api/dashboard/{id}/` | `DashboardService.getDashboardDetails` | ❌ EE only | |
| `PATCH /api/dashboard/{id}/widgets/{id}/` | `DashboardService.updateDashboardWidget` | ❌ EE only | |
| `GET/PATCH …/projects/{id}/epics-user-properties/` | `IssueFiltersService.fetchProjectEpicFilters` | ❌ Not in CE | Epics are EE |
| `POST …/projects/{id}/bulk-subscribe-issues/` | `IssueService.bulkSubscribeIssues` | ❌ Not in CE | |
| `GET /api/workspaces/{slug}/importers/jira` | `JiraImporterService` | ❌ Jira EE only | |
| `POST …/projects/importers/jira/` | `JiraImporterService` | ❌ Jira EE only | |
| `GET /api/configs/` | `AppConfigService.envConfig` | ❌ Not in CE | May be served by Next.js SSR layer |

### 3c. Missing in Rust — Space/Publish Board App (Critical for `apps/space`)

The **Space** app (public issue boards / publish boards) depends entirely on
`/api/public/anchor/{anchor}/...` endpoints. These are in Django CE (`plane.space.urls`)
but **not yet ported to Rust**.

The Space app is currently **non-functional** when pointed at the Rust backend.

#### Django CE space endpoints NOT in Rust:

| Method | Path | Django View |
|--------|------|-------------|
| GET | `/api/public/anchor/{anchor}/meta/` | `ProjectMetaDataEndpoint` |
| GET | `/api/public/anchor/{anchor}/settings/` | `ProjectDeployBoardPublicSettingsEndpoint` |
| GET | `/api/public/anchor/{anchor}/issues/` | `ProjectIssuesPublicEndpoint` |
| GET | `/api/public/workspaces/{slug}/projects/{id}/anchor/` | `WorkspaceProjectAnchorEndpoint` |
| GET | `/api/public/anchor/{anchor}/cycles/` | `ProjectCyclesEndpoint` |
| GET | `/api/public/anchor/{anchor}/modules/` | `ProjectModulesEndpoint` |
| GET | `/api/public/anchor/{anchor}/states/` | `ProjectStatesEndpoint` |
| GET | `/api/public/anchor/{anchor}/labels/` | `ProjectLabelsEndpoint` |
| GET | `/api/public/anchor/{anchor}/members/` | `ProjectMembersEndpoint` |
| GET | `/api/public/anchor/{anchor}/issues/{id}/` | `IssueRetrievePublicEndpoint` |
| GET/POST | `/api/public/anchor/{anchor}/issues/{id}/comments/` | `IssueCommentPublicViewSet` |
| GET/PATCH/DELETE | `/api/public/anchor/{anchor}/issues/{id}/comments/{id}/` | `IssueCommentPublicViewSet` |
| GET/POST | `/api/public/anchor/{anchor}/issues/{id}/reactions/` | `IssueReactionPublicViewSet` |
| DELETE | `/api/public/anchor/{anchor}/issues/{id}/reactions/{code}/` | `IssueReactionPublicViewSet` |
| GET/POST | `/api/public/anchor/{anchor}/comments/{id}/reactions/` | `CommentReactionPublicViewSet` |
| DELETE | `/api/public/anchor/{anchor}/comments/{id}/reactions/{code}/` | `CommentReactionPublicViewSet` |
| GET/POST/DELETE | `/api/public/anchor/{anchor}/issues/{id}/votes/` | `IssueVotePublicViewSet` |
| GET/POST | `/api/public/anchor/{anchor}/intakes/{id}/intake-issues/` | `IntakeIssuePublicViewSet` |
| GET/PATCH/DELETE | `/api/public/anchor/{anchor}/intakes/{id}/intake-issues/{id}/` | `IntakeIssuePublicViewSet` |
| POST | `/api/public/assets/v2/anchor/{anchor}/` | `EntityAssetEndpoint` |
| GET/PATCH/DELETE | `/api/public/assets/v2/anchor/{anchor}/{pk}/` | `EntityAssetEndpoint` |
| POST | `/api/public/assets/v2/anchor/{anchor}/restore/{pk}/` | `AssetRestoreEndpoint` |
| POST | `/api/public/assets/v2/anchor/{anchor}/{entity_id}/bulk/` | `EntityBulkAssetEndpoint` |

**Action:** File a bead to implement the Space public board API in Rust.

### 3d. Pending / Low Priority

| Endpoint | Called By | Notes |
|----------|-----------|-------|
| `GET/POST/DELETE …/workspace-integrations/{id}/project-slack-sync/` | `AppInstallationService` | Entity exists; handler not yet implemented. Low priority (Slack CE integration rare). |

---

## 4. Smoke Test Plan

> **Note:** A live smoke test against a running Rust instance was not executed in this audit
> (no browser/server available in the audit environment). The following flows should be
> validated manually against `http://localhost:8000` with `VITE_API_BASE_URL=http://localhost:8000`.

### Core Flows to Validate

| Flow | Key Endpoints | Expected Result |
|------|--------------|-----------------|
| **Login** | `POST /auth/sign-in`, `GET /api/users/me/` | Session cookie set; user data returned |
| **Workspace load** | `GET /api/workspaces/`, `GET /api/users/me/settings` | Workspace list visible |
| **Project load** | `GET /api/workspaces/{slug}/projects/` | Project list visible |
| **Issue list** | `GET /api/workspaces/{slug}/projects/{id}/issues/` | Issues visible |
| **Create issue** | `POST /api/workspaces/{slug}/projects/{id}/issues/` | Issue created |
| **Issue detail** | `GET /api/workspaces/{slug}/projects/{id}/issues/{id}/` | All fields present |
| **Issue comment** | `POST/GET …/issues/{id}/comments/` | Comment posted and shown |
| **Page view** | `GET …/projects/{id}/pages/` | Pages listed |
| **Cycle view** | `GET …/projects/{id}/cycles/` | Cycles listed |
| **Module view** | `GET …/projects/{id}/modules/` | Modules listed |
| **Intake/Inbox** | `GET …/projects/{id}/intakes/` | Intake visible |
| **Search** | `GET /api/workspaces/{slug}/search?query=...` | Results returned |
| **Notifications** | `GET …/users/notifications/unread` | Count returned |

### Known Non-functional Flows (Gaps)

| Flow | Reason |
|------|--------|
| Space/Publish board (`apps/space`) | `/api/public/anchor/…` not in Rust |
| Home Dashboard widgets | EE-only endpoints |
| My Issues view | EE-only |
| Epics filter panel | EE-only `epics-user-properties` |

---

## 5. Summary

| Category | Count | Verdict |
|----------|-------|---------|
| Endpoints validated ✅ | ~200+ | All major CE endpoints in Rust |
| FE path bugs (wrong URL) | 2 | FE fix needed, not Rust |
| EE-only (deferred) | 9 | No action needed |
| Missing in Rust — Space app | 23 | **Critical: Space app broken** |
| Pending low priority | 1 | Slack sync |

**Overall verdict:** The `apps/web` + `apps/admin` frontends point correctly to api_rust and
all CE features are implemented. The **`apps/space` publish-board app has a critical gap** —
23 public/anchor endpoints need to be ported from Django CE `plane.space` to Rust.

Two frontend path bugs were identified (`issue-display-properties/` and `cycle-progress/`)
that cause 404s regardless of whether Django or Rust is the backend.
