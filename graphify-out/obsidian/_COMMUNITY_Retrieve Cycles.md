---
type: community
cohesion: 1.00
members: 1
---

# Retrieve Cycles

**Cohesion:** 1.00 - tightly connected
**Members:** 1 nodes

## Members

- [[List or retrieve cycles          Retrieve all cycles in a project or get details]] - rationale - api/plane/api/views/cycle.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Retrieve_Cycles
SORT file.name ASC
```
