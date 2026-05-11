---
type: community
cohesion: 0.83
members: 4
---

# Sync Signals

**Cohesion:** 0.83 - tightly connected
**Members:** 4 nodes

## Members

- [[_table_exists()]] - code - api/plane/db/models/integration/signals.py
- [[signals.py]] - code - api/plane/db/models/integration/signals.py
- [[sync_comment_to_external()]] - code - api/plane/db/models/integration/signals.py
- [[sync_issue_to_external()]] - code - api/plane/db/models/integration/signals.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Sync_Signals
SORT file.name ASC
```
