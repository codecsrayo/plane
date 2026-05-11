---
type: community
cohesion: 0.24
members: 11
---

# Community 450

**Cohesion:** 0.24 - loosely connected
**Members:** 11 nodes

## Members
- [[Function takes in two json and computes differences between keys of both the jso]] - rationale - api/plane/bgtasks/webhook_task.py
- [[Process and send webhook notifications for various activities in the system.]] - rationale - api/plane/bgtasks/webhook_task.py
- [[Retrieve and serialize model data based on the event type.      Args         ev]] - rationale - api/plane/bgtasks/webhook_task.py
- [[Send webhook notifications to configured endpoints.      Args         webhook (]] - rationale - api/plane/bgtasks/webhook_task.py
- [[get_issue_prefetches()]] - code - api/plane/bgtasks/webhook_task.py
- [[get_model_data()]] - code - api/plane/bgtasks/webhook_task.py
- [[model_activity()]] - code - api/plane/bgtasks/webhook_task.py
- [[save_webhook_log()]] - code - api/plane/bgtasks/webhook_task.py
- [[webhook_activity()]] - code - api/plane/bgtasks/webhook_task.py
- [[webhook_send_task()]] - code - api/plane/bgtasks/webhook_task.py
- [[webhook_task.py]] - code - api/plane/bgtasks/webhook_task.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Community_450
SORT file.name ASC
```

## Connections to other communities
- 4 edges to [[_COMMUNITY_Community 48]]
- 1 edge to [[_COMMUNITY_Community 123]]

## Top bridge nodes
- [[webhook_activity()]] - degree 5, connects to 2 communities
- [[webhook_task.py]] - degree 7, connects to 1 community
- [[webhook_send_task()]] - degree 4, connects to 1 community
- [[save_webhook_log()]] - degree 3, connects to 1 community