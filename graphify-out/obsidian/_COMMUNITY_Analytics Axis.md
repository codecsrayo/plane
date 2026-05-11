---
type: community
cohesion: 0.29
members: 7
---

# Analytics Axis

**Cohesion:** 0.29 - loosely connected
**Members:** 7 nodes

## Members

- [[ANALYTICS_DURATION_FILTER_OPTIONS]] - code - constants/src/analytics/common.ts
- [[ANALYTICS_INSIGHTS_FIELDS]] - code - constants/src/analytics/common.ts
- [[ANALYTICS_V2_DATE_KEYS]] - code - constants/src/analytics/common.ts
- [[ANALYTICS_X_AXIS_VALUES]] - code - constants/src/analytics/common.ts
- [[ANALYTICS_Y_AXIS_VALUES]] - code - constants/src/analytics/common.ts
- [[IInsightField]] - code - constants/src/analytics/common.ts
- [[common.ts_3]] - code - constants/src/analytics/common.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Analytics_Axis
SORT file.name ASC
```
