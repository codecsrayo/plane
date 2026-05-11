---
type: community
cohesion: 1.00
members: 1
---

# Return Repo

**Cohesion:** 1.00 - tightly connected
**Members:** 1 nodes

## Members

- [[Return all repo syncs for this workspace's GitHub integration.]] - rationale - api/plane/app/views/integration/base.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Return_Repo
SORT file.name ASC
```
