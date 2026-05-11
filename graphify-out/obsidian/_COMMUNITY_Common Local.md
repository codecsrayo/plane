---
type: community
cohesion: 0.40
members: 5
---

# Common Local

**Cohesion:** 0.40 - moderately connected
**Members:** 5 nodes

## Members

- [[common.py]] - code - api/plane/settings/common.py
- [[local.py]] - code - api/plane/settings/local.py
- [[openapi.py]] - code - api/plane/settings/openapi.py
- [[production.py]] - code - api/plane/settings/production.py
- [[test.py]] - code - api/plane/settings/test.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Common_Local
SORT file.name ASC
```
