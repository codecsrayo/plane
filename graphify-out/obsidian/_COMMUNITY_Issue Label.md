---
type: community
cohesion: 0.25
members: 8
---

# Issue Label

**Cohesion:** 0.25 - loosely connected
**Members:** 8 nodes

## Members

- [[.constructor()_103]] - code - services/issue/issue_label.service.ts
- [[.createIssueLabel()]] - code - services/issue/issue_label.service.ts
- [[.deleteIssueLabel()]] - code - services/issue/issue_label.service.ts
- [[.getProjectLabels()]] - code - services/issue/issue_label.service.ts
- [[.getWorkspaceIssueLabels()]] - code - services/issue/issue_label.service.ts
- [[.patchIssueLabel()]] - code - services/issue/issue_label.service.ts
- [[IssueLabelService]] - code - services/issue/issue_label.service.ts
- [[issue_label.service.ts]] - code - services/issue/issue_label.service.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Issue_Label
SORT file.name ASC
```

## Connections to other communities

- 2 edges to [[_COMMUNITY_API Services]]
- 1 edge to [[_COMMUNITY_Issue Layouts]]

## Top bridge nodes

- [[issue_label.service.ts]] - degree 3, connects to 1 community
- [[.getProjectLabels()]] - degree 2, connects to 1 community
