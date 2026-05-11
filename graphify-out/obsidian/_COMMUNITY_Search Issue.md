---
type: community
cohesion: 1.00
members: 2
---

# Search Issue

**Cohesion:** 1.00 - tightly connected
**Members:** 2 nodes

## Members

- [[issue_search.py]] - code - api/plane/utils/issue_search.py
- [[search_issues()]] - code - api/plane/utils/issue_search.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Search_Issue
SORT file.name ASC
```
