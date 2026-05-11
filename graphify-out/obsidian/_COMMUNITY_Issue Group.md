---
type: community
cohesion: 0.13
members: 17
---

# Issue Group

**Cohesion:** 0.13 - loosely connected
**Members:** 17 nodes

## Members

- [[.constructor()_15]] - code - store/state.store.ts
- [[.groupedProjectStates()]] - code - store/state.store.ts
- [[.projectStates()]] - code - store/state.store.ts
- [[.workspaceStates()]] - code - store/state.store.ts
- [[EIssueGroupedAction]] - code - store/issue/helpers/base-issues.store.ts
- [[ISSUE_GROUP_BY_KEY]] - code - store/issue/helpers/base-issues.store.ts
- [[StateStore]] - code - store/state.store.ts
- [[base-issues-utils.ts]] - code - store/issue/helpers/base-issues-utils.ts
- [[checkIssueDateFilter()]] - code - store/issue/helpers/base-issues-utils.ts
- [[getDifference()]] - code - store/issue/helpers/base-issues-utils.ts
- [[getGroupIssueKeyActions()]] - code - store/issue/helpers/base-issues-utils.ts
- [[getGroupedWorkItemIds()]] - code - store/issue/helpers/base-issues-utils.ts
- [[getIssueIds()]] - code - store/issue/helpers/base-issues-utils.ts
- [[getOrderedWorkItems()]] - code - store/issue/helpers/base-issues-utils.ts
- [[getPreviousIssuesState()]] - code - store/issue/helpers/base-issues-utils.ts
- [[getSortOrderToFilterEmptyValues()]] - code - store/issue/helpers/base-issues-utils.ts
- [[groupBy()]] - code - store/issue/helpers/base-issues.store.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Issue_Group
SORT file.name ASC
```

## Connections to other communities

- 10 edges to [[_COMMUNITY_Issue Store Ops]]
- 4 edges to [[_COMMUNITY_Issue Work]]
- 2 edges to [[_COMMUNITY_Workspace Draft]]
- 1 edge to [[_COMMUNITY_State Project]]
- 1 edge to [[_COMMUNITY_Spreadsheet Filters]]
- 1 edge to [[_COMMUNITY_Issue Workspace]]

## Top bridge nodes

- [[base-issues-utils.ts]] - degree 18, connects to 3 communities
- [[getIssueIds()]] - degree 4, connects to 2 communities
- [[StateStore]] - degree 5, connects to 1 community
- [[getGroupedWorkItemIds()]] - degree 4, connects to 1 community
- [[getOrderedWorkItems()]] - degree 4, connects to 1 community
