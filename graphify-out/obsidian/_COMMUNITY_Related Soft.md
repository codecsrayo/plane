---
type: community
cohesion: 0.40
members: 5
---

# Related Soft

**Cohesion:** 0.40 - moderately connected
**Members:** 5 nodes

## Members

- [[Soft delete related objects for a given model instance]] - rationale - api/plane/bgtasks/deletion_task.py
- [[deletion_task.py]] - code - api/plane/bgtasks/deletion_task.py
- [[hard_delete()]] - code - api/plane/bgtasks/deletion_task.py
- [[restore_related_objects()]] - code - api/plane/bgtasks/deletion_task.py
- [[soft_delete_related_objects()]] - code - api/plane/bgtasks/deletion_task.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Related_Soft
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Test Validate]]

## Top bridge nodes

- [[soft_delete_related_objects()]] - degree 3, connects to 1 community
