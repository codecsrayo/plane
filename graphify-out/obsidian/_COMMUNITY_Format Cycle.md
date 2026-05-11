---
type: community
cohesion: 0.27
members: 12
---

# Format Cycle

**Cohesion:** 0.27 - loosely connected
**Members:** 12 nodes

## Members

- [[calculateCycleProgress()]] - code - utils/src/cycle.ts
- [[cycle.ts_2]] - code - utils/src/cycle.ts
- [[findHowManyDaysLeft()]] - code - utils/src/datetime.ts
- [[findTotalDaysInRange()]] - code - utils/src/datetime.ts
- [[formatActiveCycle()]] - code - utils/src/cycle.ts
- [[formatV1Data()]] - code - utils/src/cycle.ts
- [[formatV2Data()]] - code - utils/src/cycle.ts
- [[generateDateArray()]] - code - utils/src/datetime.ts
- [[getScope()]] - code - utils/src/cycle.ts
- [[ideal()]] - code - utils/src/cycle.ts
- [[orderCycles()]] - code - utils/src/cycle.ts
- [[shouldFilterCycle()]] - code - utils/src/cycle.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Format_Cycle
SORT file.name ASC
```

## Connections to other communities

- 7 edges to [[_COMMUNITY_Date Convert]]
- 2 edges to [[_COMMUNITY_Order Should]]

## Top bridge nodes

- [[cycle.ts_2]] - degree 14, connects to 2 communities
- [[findTotalDaysInRange()]] - degree 5, connects to 1 community
- [[generateDateArray()]] - degree 5, connects to 1 community
- [[findHowManyDaysLeft()]] - degree 2, connects to 1 community
