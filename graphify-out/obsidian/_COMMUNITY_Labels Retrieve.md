---
type: community
cohesion: 1.00
members: 1
---

# Labels Retrieve

**Cohesion:** 1.00 - tightly connected
**Members:** 1 nodes

## Members

- [[List labels          Retrieve all labels in the project.]] - rationale - api/plane/api/views/issue.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Labels_Retrieve
SORT file.name ASC
```
