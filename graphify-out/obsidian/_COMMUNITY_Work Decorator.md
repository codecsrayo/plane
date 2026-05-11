---
type: community
cohesion: 1.00
members: 2
---

# Work Decorator

**Cohesion:** 1.00 - tightly connected
**Members:** 2 nodes

## Members

- [[Decorator for work item endpoints (main issue operations)]] - rationale - api/plane/utils/openapi/decorators.py
- [[work_item_docs()]] - code - api/plane/utils/openapi/decorators.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Work_Decorator
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Sample Schema]]
- 1 edge to [[_COMMUNITY_Decorator Endpoints]]

## Top bridge nodes

- [[work_item_docs()]] - degree 3, connects to 2 communities
