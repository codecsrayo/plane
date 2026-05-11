---
type: community
cohesion: 1.00
members: 1
---

# Github Repository

**Cohesion:** 1.00 - tightly connected
**Members:** 1 nodes

## Members

- [[Delete a GithubRepositorySync (and its orphaned GithubRepository) by id.]] - rationale - api/plane/app/views/integration/base.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Github_Repository
SORT file.name ASC
```
