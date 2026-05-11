---
type: community
cohesion: 0.16
members: 18
---

# Issues Search

**Cohesion:** 0.16 - loosely connected
**Members:** 18 nodes

## Members

- [[.exclude_issues_in_cycles()]] - code - api/plane/app/views/search/issue.py
- [[.exclude_issues_in_module()]] - code - api/plane/app/views/search/issue.py
- [[.filter_issues_by_project()]] - code - api/plane/app/views/search/issue.py
- [[.filter_issues_excluding_related_issues()]] - code - api/plane/app/views/search/issue.py
- [[.filter_issues_without_target_date()]] - code - api/plane/app/views/search/issue.py
- [[.filter_root_issues_only()]] - code - api/plane/app/views/search/issue.py
- [[.get()_36]] - code - api/plane/app/views/search/issue.py
- [[.search_issues_and_excluding_parent()]] - code - api/plane/app/views/search/issue.py
- [[.search_issues_by_query()]] - code - api/plane/app/views/search/issue.py
- [[Exclude issues in a module]] - rationale - api/plane/app/views/search/issue.py
- [[Exclude issues in cycles]] - rationale - api/plane/app/views/search/issue.py
- [[Filter issues by project]] - rationale - api/plane/app/views/search/issue.py
- [[Filter issues excluding related issues]] - rationale - api/plane/app/views/search/issue.py
- [[Filter issues without a target date]] - rationale - api/plane/app/views/search/issue.py
- [[Filter root issues only]] - rationale - api/plane/app/views/search/issue.py
- [[IssueSearchEndpoint]] - code - api/plane/app/views/search/issue.py
- [[Search issues and epics by query excluding the parent]] - rationale - api/plane/app/views/search/issue.py
- [[Search issues by query]] - rationale - api/plane/app/views/search/issue.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Issues_Search
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Partial Endpoint]]
- 1 edge to [[_COMMUNITY_Endpoint User]]
- 1 edge to [[_COMMUNITY_Endpoint User]]
- 1 edge to [[_COMMUNITY_Endpoint Workspace]]

## Top bridge nodes

- [[IssueSearchEndpoint]] - degree 12, connects to 3 communities
- [[.get()_36]] - degree 10, connects to 1 community
