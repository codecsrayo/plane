---
type: community
cohesion: 1.00
members: 1
---

# Issue Activities

**Cohesion:** 1.00 - tightly connected
**Members:** 1 nodes

## Members

- [[List issue activities          Retrieve chronological activity logs for an issue]] - rationale - api/plane/api/views/issue.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Issue_Activities
SORT file.name ASC
```
