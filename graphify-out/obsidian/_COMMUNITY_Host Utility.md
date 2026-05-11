---
type: community
cohesion: 0.40
members: 6
---

# Host Utility

**Cohesion:** 0.40 - moderately connected
**Members:** 6 nodes

## Members

- [[Utility function to return host  origin from the request_1]] - rationale - api/plane/authentication/utils/host.py
- [[Utility function to return host  origin from the request]] - rationale - api/plane/utils/host.py
- [[base_host()]] - code - api/plane/authentication/utils/host.py
- [[host.py_1]] - code - api/plane/authentication/utils/host.py
- [[host.py]] - code - api/plane/utils/host.py
- [[user_ip()]] - code - api/plane/authentication/utils/host.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Host_Utility
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Test Validate]]
- 1 edge to [[_COMMUNITY_Middleware Logger]]

## Top bridge nodes

- [[base_host()]] - degree 5, connects to 1 community
- [[user_ip()]] - degree 3, connects to 1 community
