---
type: community
cohesion: 1.00
members: 2
---

# Issue Comment

**Cohesion:** 1.00 - tightly connected
**Members:** 2 nodes

## Members

- [[Decorator for issue comment endpoints]] - rationale - api/plane/utils/openapi/decorators.py
- [[issue_comment_docs()]] - code - api/plane/utils/openapi/decorators.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Issue_Comment
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Sample Schema]]
- 1 edge to [[_COMMUNITY_Decorator Endpoints]]

## Top bridge nodes

- [[issue_comment_docs()]] - degree 3, connects to 2 communities
