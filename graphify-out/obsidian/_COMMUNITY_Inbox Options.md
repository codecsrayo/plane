---
type: community
cohesion: 0.33
members: 6
---

# Inbox Options

**Cohesion:** 0.33 - loosely connected
**Members:** 6 nodes

## Members

- [[EPastDurationFilters]] - code - constants/src/intake.ts
- [[INBOX_ISSUE_ORDER_BY_OPTIONS]] - code - constants/src/intake.ts
- [[INBOX_ISSUE_SORT_BY_OPTIONS]] - code - constants/src/intake.ts
- [[INBOX_STATUS]] - code - constants/src/intake.ts
- [[PAST_DURATION_FILTER_OPTIONS]] - code - constants/src/intake.ts
- [[intake.ts]] - code - constants/src/intake.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Inbox_Options
SORT file.name ASC
```
