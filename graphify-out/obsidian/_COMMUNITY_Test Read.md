---
type: community
cohesion: 1.00
members: 1
---

# Test Read

**Cohesion:** 1.00 - tightly connected
**Members:** 1 nodes

## Members

- [[Test __call__ with read methods waits for process_view.]] - rationale - api/plane/tests/unit/middleware/test_db_routing.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Test_Read
SORT file.name ASC
```
