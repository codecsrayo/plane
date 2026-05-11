---
type: community
cohesion: 0.24
members: 10
---

# Installation Access

**Cohesion:** 0.24 - loosely connected
**Members:** 10 nodes

## Members

- [[AppClaims]] - code - api_rust/src/utils/github_app.rs
- [[Exchange a GitHub App installation_id for a short-lived installation access toke]] - rationale - api/plane/utils/github_app.py
- [[GitHub App authentication helpers.  Usage     from plane.utils.github_app impor]] - rationale - api/plane/utils/github_app.py
- [[Read a value using the same InstanceConfigurationenv resolution as the rest of]] - rationale - api/plane/utils/github_app.py
- [[Return the installation access token plus a machine-readable error string.]] - rationale - api/plane/utils/github_app.py
- [[_get_config_value()]] - code - api/plane/utils/github_app.py
- [[get_installation_access_token()]] - code - api_rust/src/utils/github_app.rs
- [[get_installation_access_token_result()]] - code - api/plane/utils/github_app.py
- [[github_app.py]] - code - api/plane/utils/github_app.py
- [[github_app.rs]] - code - api_rust/src/utils/github_app.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Installation_Access
SORT file.name ASC
```

## Connections to other communities

- 2 edges to [[_COMMUNITY_Session Presigned]]
- 1 edge to [[_COMMUNITY_Endpoint Repositories]]
- 1 edge to [[_COMMUNITY_Endpoint Magic]]

## Top bridge nodes

- [[get_installation_access_token()]] - degree 5, connects to 1 community
- [[get_installation_access_token_result()]] - degree 5, connects to 1 community
- [[_get_config_value()]] - degree 4, connects to 1 community
- [[github_app.rs]] - degree 3, connects to 1 community
