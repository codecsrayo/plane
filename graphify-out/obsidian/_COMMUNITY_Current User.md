---
type: community
cohesion: 1.00
members: 1
---

# Current User

**Cohesion:** 1.00 - tightly connected
**Members:** 1 nodes

## Members

- [[Get current user          Retrieve the authenticated user's profile information]] - rationale - api/plane/api/views/user.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Current_User
SORT file.name ASC
```
