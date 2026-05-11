---
type: community
cohesion: 1.00
members: 1
---

# States Retrieve

**Cohesion:** 1.00 - tightly connected
**Members:** 1 nodes

## Members

- [[List states          Retrieve all workflow states for a project.         Returns]] - rationale - api/plane/api/views/state.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/States_Retrieve
SORT file.name ASC
```
