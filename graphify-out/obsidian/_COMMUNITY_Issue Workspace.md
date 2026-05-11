---
type: community
cohesion: 0.29
members: 7
---

# Issue Workspace

**Cohesion:** 0.29 - loosely connected
**Members:** 7 nodes

## Members

- [[.archivedEstimateIds()]] - code - store/estimates/project-estimate.store.ts
- [[.constructor()_37]] - code - store/issue/workspace-draft/issue.store.ts
- [[.issueIds()]] - code - store/issue/workspace-draft/issue.store.ts
- [[.updateWorkspaceUserDraftIssueCount()]] - code - store/issue/workspace-draft/issue.store.ts
- [[WorkspaceDraftIssues]] - code - store/issue/workspace-draft/issue.store.ts
- [[orderBy()]] - code - store/issue/helpers/base-issues.store.ts
- [[populateIssueDataForSorting()]] - code - store/issue/helpers/base-issues.store.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Issue_Workspace
SORT file.name ASC
```

## Connections to other communities

- 2 edges to [[_COMMUNITY_Issue Store Ops]]
- 1 edge to [[_COMMUNITY_Link Home]]
- 1 edge to [[_COMMUNITY_Issue Draft]]
- 1 edge to [[_COMMUNITY_Issue Group]]
- 1 edge to [[_COMMUNITY_Estimate Project]]

## Top bridge nodes

- [[orderBy()]] - degree 6, connects to 3 communities
- [[WorkspaceDraftIssues]] - degree 4, connects to 1 community
- [[.archivedEstimateIds()]] - degree 2, connects to 1 community
- [[populateIssueDataForSorting()]] - degree 2, connects to 1 community
