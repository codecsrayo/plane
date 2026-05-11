---
type: community
cohesion: 0.22
members: 9
---

# Password Returns

**Cohesion:** 0.22 - loosely connected
**Members:** 9 nodes

## Members

- [[auth_password.rs]] - code - api_rust/tests/auth_password.rs
- [[change_password_without_session_returns_401()]] - code - api_rust/tests/auth_password.rs
- [[forgot_password_unknown_user_returns_400()]] - code - api_rust/tests/auth_password.rs
- [[forgot_password_with_smtp_rejects_invalid_email()]] - code - api_rust/tests/auth_password.rs
- [[forgot_password_without_smtp_returns_400()]] - code - api_rust/tests/auth_password.rs
- [[location_has_error()_1]] - code - api_rust/tests/auth_password.rs
- [[reset_password_expired_token_redirects_with_error()]] - code - api_rust/tests/auth_password.rs
- [[reset_password_invalid_uidb64_redirects_with_error()]] - code - api_rust/tests/auth_password.rs
- [[set_password_without_session_returns_401()]] - code - api_rust/tests/auth_password.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Password_Returns
SORT file.name ASC
```

## Connections to other communities

- 7 edges to [[_COMMUNITY_Returns Sign]]

## Top bridge nodes

- [[change_password_without_session_returns_401()]] - degree 2, connects to 1 community
- [[forgot_password_unknown_user_returns_400()]] - degree 2, connects to 1 community
- [[forgot_password_with_smtp_rejects_invalid_email()]] - degree 2, connects to 1 community
- [[forgot_password_without_smtp_returns_400()]] - degree 2, connects to 1 community
- [[reset_password_expired_token_redirects_with_error()]] - degree 2, connects to 1 community
