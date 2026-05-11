---
type: community
cohesion: 0.14
members: 14
---

# Returns Without

**Cohesion:** 0.14 - loosely connected
**Members:** 14 nodes

## Members

- [[auth_oauth.rs]] - code - api_rust/tests/auth_oauth.rs
- [[gitea_callback_without_code_returns_400()]] - code - api_rust/tests/auth_oauth.rs
- [[gitea_initiate_without_client_id_returns_400()]] - code - api_rust/tests/auth_oauth.rs
- [[github_app_callback_with_unknown_workspace_returns_error_html()]] - code - api_rust/tests/auth_oauth.rs
- [[github_app_callback_without_installation_id_returns_error_html()]] - code - api_rust/tests/auth_oauth.rs
- [[github_user_callback_without_auth_returns_401()]] - code - api_rust/tests/auth_oauth.rs
- [[gitlab_callback_without_code_returns_400()]] - code - api_rust/tests/auth_oauth.rs
- [[gitlab_callback_without_state_returns_400()]] - code - api_rust/tests/auth_oauth.rs
- [[gitlab_initiate_with_client_id_redirects_to_gitlab()]] - code - api_rust/tests/auth_oauth.rs
- [[gitlab_initiate_without_client_id_returns_400()]] - code - api_rust/tests/auth_oauth.rs
- [[google_callback_without_code_returns_400()]] - code - api_rust/tests/auth_oauth.rs
- [[google_callback_without_cookie_returns_html_error()]] - code - api_rust/tests/auth_oauth.rs
- [[google_initiate_with_client_id_redirects_to_google()]] - code - api_rust/tests/auth_oauth.rs
- [[google_initiate_without_client_id_returns_400()]] - code - api_rust/tests/auth_oauth.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Returns_Without
SORT file.name ASC
```

## Connections to other communities

- 13 edges to [[_COMMUNITY_Returns Sign]]

## Top bridge nodes

- [[gitea_callback_without_code_returns_400()]] - degree 2, connects to 1 community
- [[gitea_initiate_without_client_id_returns_400()]] - degree 2, connects to 1 community
- [[github_app_callback_with_unknown_workspace_returns_error_html()]] - degree 2, connects to 1 community
- [[github_app_callback_without_installation_id_returns_error_html()]] - degree 2, connects to 1 community
- [[github_user_callback_without_auth_returns_401()]] - degree 2, connects to 1 community
