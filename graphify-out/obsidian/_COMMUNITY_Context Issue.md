---
type: community
cohesion: 1.00
members: 1
---

# Context Issue

**Cohesion:** 1.00 - tightly connected
**Members:** 1 nodes

## Members

- [[Get context data for issue serialization.]] - rationale - api/plane/utils/exporters/schemas/issue.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Context_Issue
SORT file.name ASC
```
