---
type: community
cohesion: 1.00
members: 1
---

# User Profile

**Cohesion:** 1.00 - tightly connected
**Members:** 1 nodes

## Members

- [[Delete user asset.          Delete a user profile asset (avatar or cover image)]] - rationale - api/plane/api/views/asset.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/User_Profile
SORT file.name ASC
```
