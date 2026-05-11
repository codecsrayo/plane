---
type: community
cohesion: 1.00
members: 1
---

# Projects Retrieve

**Cohesion:** 1.00 - tightly connected
**Members:** 1 nodes

## Members

- [[List projects          Retrieve all projects in a workspace or get details of a]] - rationale - api/plane/api/views/project.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Projects_Retrieve
SORT file.name ASC
```
