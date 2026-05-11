---
type: community
cohesion: 0.33
members: 6
---

# Inbox Project

**Cohesion:** 0.33 - loosely connected
**Members:** 6 nodes

## Members

- [[.constructor()_64]] - code - store/inbox/project-inbox.store.ts
- [[.filteredInboxIssueIds()]] - code - store/inbox/project-inbox.store.ts
- [[.getAppliedFiltersCount()]] - code - store/inbox/project-inbox.store.ts
- [[.inboxFilters()]] - code - store/inbox/project-inbox.store.ts
- [[.inboxSorting()]] - code - store/inbox/project-inbox.store.ts
- [[ProjectInboxStore]] - code - store/inbox/project-inbox.store.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Inbox_Project
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Project Root Store]]
- 1 edge to [[_COMMUNITY_Inbox Issue]]

## Top bridge nodes

- [[ProjectInboxStore]] - degree 7, connects to 2 communities
