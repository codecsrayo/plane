---
type: community
cohesion: 0.14
members: 33
---

# Path Redirection

**Cohesion:** 0.14 - loosely connected
**Members:** 33 nodes

## Members

- [[.get()_50]] - code - api/plane/authentication/views/app/gitea.py
- [[.get()_46]] - code - api/plane/authentication/views/app/gitlab.py
- [[.get()_45]] - code - api/plane/authentication/views/app/gitlab.py
- [[.get()_44]] - code - api/plane/authentication/views/app/google.py
- [[.get()_58]] - code - api/plane/authentication/views/space/gitea.py
- [[.get()_57]] - code - api/plane/authentication/views/space/gitea.py
- [[.get()_56]] - code - api/plane/authentication/views/space/github.py
- [[.get()_54]] - code - api/plane/authentication/views/space/gitlab.py
- [[.get()_53]] - code - api/plane/authentication/views/space/gitlab.py
- [[.get()_52]] - code - api/plane/authentication/views/space/google.py
- [[.get()_51]] - code - api/plane/authentication/views/space/google.py
- [[.post()_25]] - code - api/plane/authentication/views/app/email.py
- [[.post()_26]] - code - api/plane/authentication/views/app/email.py
- [[.post()_23]] - code - api/plane/authentication/views/app/magic.py
- [[.post()_24]] - code - api/plane/authentication/views/app/magic.py
- [[.post()_34]] - code - api/plane/authentication/views/space/email.py
- [[.post()_35]] - code - api/plane/authentication/views/space/email.py
- [[.post()_32]] - code - api/plane/authentication/views/space/magic.py
- [[.post()_33]] - code - api/plane/authentication/views/space/magic.py
- [[.validate_email()]] - code - api/plane/api/serializers/invite.py
- [[Check for suspicious patterns that might indicate malicious intent.      Args]] - rationale - api/plane/utils/path_validator.py
- [[Get the allowed hosts from the settings.]] - rationale - api/plane/utils/path_validator.py
- [[Safely construct a redirect URL with validated next_path.      Args         bas]] - rationale - api/plane/utils/path_validator.py
- [[Validates that next_path is a safe relative path for redirection.]] - rationale - api/plane/utils/path_validator.py
- [[_contains_suspicious_patterns()]] - code - api/plane/utils/path_validator.py
- [[get_allowed_hosts()]] - code - api/plane/utils/path_validator.py
- [[get_redirection_path()]] - code - api/plane/authentication/utils/redirection_path.py
- [[get_safe_redirect_url()]] - code - api/plane/utils/path_validator.py
- [[login.py]] - code - api/plane/authentication/utils/login.py
- [[path_validator.py]] - code - api/plane/utils/path_validator.py
- [[redirection_path.py]] - code - api/plane/authentication/utils/redirection_path.py
- [[user_login()]] - code - api/plane/authentication/utils/login.py
- [[validate_next_path()]] - code - api/plane/utils/path_validator.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Path_Redirection
SORT file.name ASC
```

## Connections to other communities

- 33 edges to [[_COMMUNITY_Endpoint Magic]]
- 20 edges to [[_COMMUNITY_Endpoint Sign]]
- 8 edges to [[_COMMUNITY_Gitea Endpoint]]
- 8 edges to [[_COMMUNITY_Google Endpoint]]
- 8 edges to [[_COMMUNITY_Endpoint Github]]
- 2 edges to [[_COMMUNITY_Endpoint User]]
- 2 edges to [[_COMMUNITY_Endpoint Common]]
- 1 edge to [[_COMMUNITY_Client Login]]
- 1 edge to [[_COMMUNITY_Test Validate]]
- 1 edge to [[_COMMUNITY_Endpoint Workspace]]
- 1 edge to [[_COMMUNITY_Endpoint State]]
- 1 edge to [[_COMMUNITY_Check User]]
- 1 edge to [[_COMMUNITY_Middleware Logger]]

## Top bridge nodes

- [[.validate_email()]] - degree 16, connects to 6 communities
- [[user_login()]] - degree 22, connects to 5 communities
- [[get_safe_redirect_url()]] - degree 26, connects to 3 communities
- [[validate_next_path()]] - degree 16, connects to 2 communities
- [[.post()_34]] - degree 8, connects to 2 communities
