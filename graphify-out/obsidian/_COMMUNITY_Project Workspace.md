---
type: community
cohesion: 1.00
members: 1
---

# Project Workspace

**Cohesion:** 1.00 - tightly connected
**Members:** 1 nodes

## Members

- [[Create project          Create a new project in the workspace with default state]] - rationale - api/plane/api/views/project.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Project_Workspace
SORT file.name ASC
```
