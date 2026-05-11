---
type: community
cohesion: 0.33
members: 6
---

# Plan Features

**Cohesion:** 0.33 - loosely connected
**Members:** 6 nodes

## Members

- [[BUSINESS_PLAN_FEATURES]] - code - constants/src/subscription.ts
- [[ENTERPRISE_PLAN_FEATURES]] - code - constants/src/subscription.ts
- [[FREE_PLAN_UPGRADE_FEATURES]] - code - constants/src/subscription.ts
- [[ONE_PLAN_FEATURES]] - code - constants/src/subscription.ts
- [[PRO_PLAN_FEATURES]] - code - constants/src/subscription.ts
- [[subscription.ts]] - code - constants/src/subscription.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Plan_Features
SORT file.name ASC
```
