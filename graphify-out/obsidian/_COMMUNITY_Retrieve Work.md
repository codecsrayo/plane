---
type: community
cohesion: 1.00
members: 1
---

# Retrieve Work

**Cohesion:** 1.00 - tightly connected
**Members:** 1 nodes

## Members

- [[List or retrieve cycle work items          Retrieve all work items assigned to a]] - rationale - api/plane/api/views/cycle.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Retrieve_Work
SORT file.name ASC
```
