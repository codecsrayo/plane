---
type: community
cohesion: 0.33
members: 6
---

# Active Cycles

**Cohesion:** 0.33 - loosely connected
**Members:** 6 nodes

## Members

- [[.constructor()_20]] - code - services/src/cycle/cycle-analytics.service.ts
- [[.workspaceActiveCyclesAnalytics()]] - code - services/src/cycle/cycle-analytics.service.ts
- [[.workspaceActiveCyclesProgress()]] - code - services/src/cycle/cycle-analytics.service.ts
- [[.workspaceActiveCyclesProgressPro()]] - code - services/src/cycle/cycle-analytics.service.ts
- [[CycleAnalyticsService]] - code - services/src/cycle/cycle-analytics.service.ts
- [[cycle-analytics.service.ts]] - code - services/src/cycle/cycle-analytics.service.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Active_Cycles
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Sites Cycle]]

## Top bridge nodes

- [[cycle-analytics.service.ts]] - degree 2, connects to 1 community
