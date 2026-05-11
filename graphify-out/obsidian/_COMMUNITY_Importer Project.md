---
type: community
cohesion: 0.40
members: 5
---

# Importer Project

**Cohesion:** 0.40 - moderately connected
**Members:** 5 nodes

## Members

- [[Import issues from a GitHub repository into a Plane project]] - rationale - api/plane/bgtasks/importer_task.py
- [[Import issues from a GitLab project into a Plane project]] - rationale - api/plane/bgtasks/importer_task.py
- [[github_importer_task()]] - code - api/plane/bgtasks/importer_task.py
- [[gitlab_importer_task()]] - code - api/plane/bgtasks/importer_task.py
- [[importer_task.py]] - code - api/plane/bgtasks/importer_task.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Importer_Project
SORT file.name ASC
```
