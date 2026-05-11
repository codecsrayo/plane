---
type: community
cohesion: 0.83
members: 4
---

# Issue Description

**Cohesion:** 0.83 - tightly connected
**Members:** 4 nodes

## Members

- [[issue_description_version_task()]] - code - api/plane/bgtasks/issue_description_version_task.py
- [[issue_description_version_task.py]] - code - api/plane/bgtasks/issue_description_version_task.py
- [[should_update_existing_version()]] - code - api/plane/bgtasks/issue_description_version_task.py
- [[update_existing_version()]] - code - api/plane/bgtasks/issue_description_version_task.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Issue_Description
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Task Object]]

## Top bridge nodes

- [[issue_description_version_task()]] - degree 4, connects to 1 community
