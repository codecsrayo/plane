---
type: community
cohesion: 0.40
members: 5
---

# Sync Comment

**Cohesion:** 0.40 - moderately connected
**Members:** 5 nodes

## Members

- [[sync_comment_to_github_task()]] - code - api/plane/bgtasks/sync_task.py
- [[sync_comment_to_gitlab_task()]] - code - api/plane/bgtasks/sync_task.py
- [[sync_issue_to_github_task()]] - code - api/plane/bgtasks/sync_task.py
- [[sync_issue_to_gitlab_task()]] - code - api/plane/bgtasks/sync_task.py
- [[sync_task.py]] - code - api/plane/bgtasks/sync_task.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Sync_Comment
SORT file.name ASC
```
