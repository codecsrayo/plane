---
type: community
cohesion: 0.09
members: 39
---

# Workspace Webhook

**Cohesion:** 0.09 - loosely connected
**Members:** 39 nodes

## Members

- [[.constructor()_188]] - code - live/src/extensions/database.ts
- [[.from_model()_10]] - code - api_rust/src/routes/webhooks.rs
- [[CreateWebhookRequest]] - code - api_rust/src/routes/webhooks.rs
- [[Database]] - code - live/src/extensions/database.ts
- [[UpdateWebhookRequest]] - code - api_rust/src/routes/webhooks.rs
- [[WebhookLogResponse]] - code - api_rust/src/routes/webhooks.rs
- [[WebhookResponse]] - code - api_rust/src/routes/webhooks.rs
- [[build_slack_metadata()]] - code - api_rust/src/routes/integrations/workspace.rs
- [[create_pr_state_mapping()]] - code - api_rust/src/routes/integrations/pr_state.rs
- [[create_webhook()_1]] - code - api_rust/src/routes/webhooks.rs
- [[create_workspace_integration()]] - code - api_rust/src/routes/integrations/workspace.rs
- [[delete_github_repo_sync()]] - code - api_rust/src/routes/integrations/github.rs
- [[delete_pr_state_mapping()]] - code - api_rust/src/routes/integrations/pr_state.rs
- [[delete_webhook()]] - code - api_rust/src/routes/webhooks.rs
- [[delete_workspace_integration()]] - code - api_rust/src/routes/integrations/workspace.rs
- [[delete_workspace_integration_by_provider()]] - code - api_rust/src/routes/integrations/workspace.rs
- [[generate_secret()]] - code - api_rust/src/routes/webhooks.rs
- [[get_or_create_api_token()]] - code - api_rust/src/routes/integrations/helpers.rs
- [[get_webhook()]] - code - api_rust/src/routes/webhooks.rs
- [[get_workspace_integration()]] - code - api_rust/src/routes/integrations/workspace.rs
- [[helpers.rs_1]] - code - api_rust/src/routes/integrations/helpers.rs
- [[list_pr_state_mappings()]] - code - api_rust/src/routes/integrations/pr_state.rs
- [[list_webhook_logs()]] - code - api_rust/src/routes/webhooks.rs
- [[list_webhooks()]] - code - api_rust/src/routes/webhooks.rs
- [[list_workspace_integrations()]] - code - api_rust/src/routes/integrations/workspace.rs
- [[permissions.rs]] - code - api_rust/src/auth/permissions.rs
- [[pr_state.rs]] - code - api_rust/src/routes/integrations/pr_state.rs
- [[provider_install()_1]] - code - api_rust/src/routes/integrations/workspace.rs
- [[regenerate_secret()]] - code - api_rust/src/routes/webhooks.rs
- [[require_role_accepts_project_role()]] - code - api_rust/src/auth/permissions.rs
- [[require_role_accepts_workspace_admin_override()]] - code - api_rust/src/auth/permissions.rs
- [[require_role_rejects_insufficient_roles()]] - code - api_rust/src/auth/permissions.rs
- [[require_workspace_admin()]] - code - api_rust/src/auth/permissions.rs
- [[require_workspace_member()]] - code - api_rust/src/auth/permissions.rs
- [[update_webhook()]] - code - api_rust/src/routes/webhooks.rs
- [[update_workspace_integration()]] - code - api_rust/src/routes/integrations/workspace.rs
- [[validate_webhook_url()]] - code - api_rust/src/routes/webhooks.rs
- [[webhooks.rs_1]] - code - api_rust/src/routes/webhooks.rs
- [[workspace.rs]] - code - api_rust/src/routes/integrations/workspace.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Workspace_Webhook
SORT file.name ASC
```

## Connections to other communities

- 18 edges to [[_COMMUNITY_User Github]]
- 13 edges to [[_COMMUNITY_Asset Issue]]
- 10 edges to [[_COMMUNITY_Issue Request]]
- 5 edges to [[_COMMUNITY_Project Response]]
- 5 edges to [[_COMMUNITY_Analytics Advance]]
- 2 edges to [[_COMMUNITY_Issue Issues]]
- 2 edges to [[_COMMUNITY_State Project]]
- 2 edges to [[_COMMUNITY_Workspace User]]
- 2 edges to [[_COMMUNITY_Request Intake]]
- 2 edges to [[_COMMUNITY_Session Presigned]]
- 1 edge to [[_COMMUNITY_Broadcast Context]]
- 1 edge to [[_COMMUNITY_Logger Title]]
- 1 edge to [[_COMMUNITY_Issue Bulk]]
- 1 edge to [[_COMMUNITY_Password Email]]

## Top bridge nodes

- [[Database]] - degree 20, connects to 9 communities
- [[require_workspace_admin()]] - degree 27, connects to 4 communities
- [[get_or_create_api_token()]] - degree 8, connects to 4 communities
- [[provider_install()_1]] - degree 8, connects to 3 communities
- [[create_webhook()_1]] - degree 8, connects to 3 communities
