---
type: community
cohesion: 0.24
members: 11
---

# Community 451

**Cohesion:** 0.24 - loosely connected
**Members:** 11 nodes

## Members
- [[Create IssueVersion object from the given issue and related data]] - rationale - api/plane/bgtasks/issue_version_sync.py
- [[Get related data for the given issue IDs]] - rationale - api/plane/bgtasks/issue_version_sync.py
- [[Get the owner ID of the issue_1]] - rationale - api/plane/bgtasks/issue_version_sync.py
- [[Task to create IssueVersion records for existing Issues in batches]] - rationale - api/plane/bgtasks/issue_version_sync.py
- [[create_issue_version()]] - code - api/plane/bgtasks/issue_version_sync.py
- [[get_owner_id()_1]] - code - api/plane/bgtasks/issue_version_sync.py
- [[get_related_data()]] - code - api/plane/bgtasks/issue_version_sync.py
- [[issue_task()]] - code - api/plane/bgtasks/issue_version_sync.py
- [[issue_version_sync.py]] - code - api/plane/bgtasks/issue_version_sync.py
- [[schedule_issue_version()]] - code - api/plane/bgtasks/issue_version_sync.py
- [[sync_issue_version()]] - code - api/plane/bgtasks/issue_version_sync.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Community_451
SORT file.name ASC
```

## Connections to other communities
- 3 edges to [[_COMMUNITY_Community 48]]
- 1 edge to [[_COMMUNITY_Community 49]]
- 1 edge to [[_COMMUNITY_Community 26]]

## Top bridge nodes
- [[create_issue_version()]] - degree 6, connects to 2 communities
- [[sync_issue_version()]] - degree 5, connects to 1 community
- [[get_related_data()]] - degree 4, connects to 1 community
- [[issue_task()]] - degree 2, connects to 1 community