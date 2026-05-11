---
type: community
cohesion: 0.17
members: 17
---

# Handle Webhook

**Cohesion:** 0.17 - loosely connected
**Members:** 17 nodes

## Members

- [[.handle_comment()]] - code - api/plane/app/views/external/sync.py
- [[.handle_issue()]] - code - api/plane/app/views/external/sync.py
- [[.handle_issue()_1]] - code - api/plane/app/views/external/sync.py
- [[.handle_merge_request()]] - code - api/plane/app/views/external/sync.py
- [[.handle_note()]] - code - api/plane/app/views/external/sync.py
- [[.handle_pull_request()]] - code - api/plane/app/views/external/sync.py
- [[.post()_6]] - code - api/plane/app/views/external/sync.py
- [[.post()_7]] - code - api/plane/app/views/external/sync.py
- [[GitHubWebhookEndpoint]] - code - api/plane/app/views/external/sync.py
- [[GitLabWebhookEndpoint]] - code - api/plane/app/views/external/sync.py
- [[Handle GitHub `issues` webhook events.          - opened   Create a Plane issue]] - rationale - api/plane/app/views/external/sync.py
- [[Handle GitHub `pull_request` webhook events and update the linked Plane issue]] - rationale - api/plane/app/views/external/sync.py
- [[Handle GitLab `merge_request` webhook events and update the linked Plane issue]] - rationale - api/plane/app/views/external/sync.py
- [[Read a webhook secret from InstanceConfiguration DB, falling back to env var.]] - rationale - api/plane/app/views/external/sync.py
- [[_get_webhook_secret()]] - code - api/plane/app/views/external/sync.py
- [[dispatch()]] - code - api/plane/app/views/external/sync.py
- [[sync.py]] - code - api/plane/app/views/external/sync.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Handle_Webhook
SORT file.name ASC
```

## Connections to other communities

- 8 edges to [[_COMMUNITY_Partial Endpoint]]
- 2 edges to [[_COMMUNITY_Endpoint User]]
- 2 edges to [[_COMMUNITY_Asset Issue]]

## Top bridge nodes

- [[.handle_pull_request()]] - degree 5, connects to 2 communities
- [[.handle_merge_request()]] - degree 5, connects to 2 communities
- [[GitHubWebhookEndpoint]] - degree 6, connects to 1 community
- [[.post()_6]] - degree 6, connects to 1 community
- [[GitLabWebhookEndpoint]] - degree 6, connects to 1 community
