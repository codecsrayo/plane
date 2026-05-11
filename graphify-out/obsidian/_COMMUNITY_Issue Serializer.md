---
type: community
cohesion: 0.70
members: 5
---

# Issue Serializer

**Cohesion:** 0.70 - tightly connected
**Members:** 5 nodes

## Members

- [[.get_assignee_ids()]] - code - api/plane/app/serializers/view.py
- [[.get_label_ids()]] - code - api/plane/app/serializers/view.py
- [[.get_module_ids()]] - code - api/plane/app/serializers/view.py
- [[.to_representation()_1]] - code - api/plane/app/serializers/view.py
- [[ViewIssueListSerializer]] - code - api/plane/app/serializers/view.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Issue_Serializer
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Issue Apply]]
- 1 edge to [[_COMMUNITY_Serializer Meta]]
- 1 edge to [[_COMMUNITY_Serializer Project]]

## Top bridge nodes

- [[ViewIssueListSerializer]] - degree 7, connects to 3 communities
