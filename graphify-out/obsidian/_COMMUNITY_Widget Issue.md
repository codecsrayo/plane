---
type: community
cohesion: 0.06
members: 36
---

# Widget Issue

**Cohesion:** 0.06 - loosely connected
**Members:** 36 nodes

## Members

- [[IIssueActivity]] - code - types/src/issues.ts
- [[TAssignedIssuesWidgetFilters]] - code - types/src/dashboard.ts
- [[TAssignedIssuesWidgetResponse]] - code - types/src/dashboard.ts
- [[TCreatedIssuesWidgetFilters]] - code - types/src/dashboard.ts
- [[TCreatedIssuesWidgetResponse]] - code - types/src/dashboard.ts
- [[TDeprecatedDashboard]] - code - types/src/dashboard.ts
- [[THomeDashboardResponse]] - code - types/src/dashboard.ts
- [[TIssue]] - code - types/src/issues/issue.ts
- [[TIssueRelation]] - code - types/src/issues/issue_relation.ts
- [[TIssueRelationIdMap]] - code - types/src/issues/issue_relation.ts
- [[TIssueRelationMap]] - code - types/src/issues/issue_relation.ts
- [[TIssueRelationTypes]] - code - types/src/issues/issue_relation.ts
- [[TIssueSubIssues]] - code - types/src/issues/issue_sub_issues.ts
- [[TIssueSubIssuesIdMap]] - code - types/src/issues/issue_sub_issues.ts
- [[TIssueSubIssuesStateDistributionMap]] - code - types/src/issues/issue_sub_issues.ts
- [[TIssuesByPriorityWidgetFilters]] - code - types/src/dashboard.ts
- [[TIssuesByPriorityWidgetResponse]] - code - types/src/dashboard.ts
- [[TIssuesByStateGroupsWidgetFilters]] - code - types/src/dashboard.ts
- [[TIssuesByStateGroupsWidgetResponse]] - code - types/src/dashboard.ts
- [[TIssuesListTypes]] - code - types/src/dashboard.ts
- [[TOverviewStatsWidgetResponse]] - code - types/src/dashboard.ts
- [[TRecentActivityWidgetResponse]] - code - types/src/dashboard.ts
- [[TRecentCollaboratorsWidgetResponse]] - code - types/src/dashboard.ts
- [[TRecentProjectsWidgetResponse]] - code - types/src/dashboard.ts
- [[TSubIssueOperations]] - code - types/src/issues/issue_sub_issues.ts
- [[TSubIssueResponse]] - code - types/src/issues/issue_sub_issues.ts
- [[TSubIssuesStateDistribution]] - code - types/src/issues/issue_sub_issues.ts
- [[TWidget]] - code - types/src/dashboard.ts
- [[TWidgetFiltersFormData]] - code - types/src/dashboard.ts
- [[TWidgetIssue]] - code - types/src/dashboard.ts
- [[TWidgetKeys]] - code - types/src/dashboard.ts
- [[TWidgetStatsRequestParams]] - code - types/src/dashboard.ts
- [[TWidgetStatsResponse]] - code - types/src/dashboard.ts
- [[dashboard.ts_1]] - code - types/src/dashboard.ts
- [[issue_relation.ts]] - code - types/src/issues/issue_relation.ts
- [[issue_sub_issues.ts]] - code - types/src/issues/issue_sub_issues.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Widget_Issue
SORT file.name ASC
```

## Connections to other communities

- 5 edges to [[_COMMUNITY_Issue Public]]
- 4 edges to [[_COMMUNITY_Issue Entity]]
- 2 edges to [[_COMMUNITY_Estimate Notification]]
- 2 edges to [[_COMMUNITY_Project State]]
- 1 edge to [[_COMMUNITY_Inbox Issue]]
- 1 edge to [[_COMMUNITY_Workspace Search]]
- 1 edge to [[_COMMUNITY_Issue Identifier]]
- 1 edge to [[_COMMUNITY_Distribution Link]]
- 1 edge to [[_COMMUNITY_Cycle Distribution]]

## Top bridge nodes

- [[TIssue]] - degree 10, connects to 7 communities
- [[dashboard.ts_1]] - degree 32, connects to 4 communities
- [[issue_sub_issues.ts]] - degree 8, connects to 1 community
- [[issue_relation.ts]] - degree 7, connects to 1 community
- [[TIssueRelationTypes]] - degree 3, connects to 1 community
