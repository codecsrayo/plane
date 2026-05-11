---
type: community
cohesion: 0.15
members: 13
---

# Magic Sign

**Cohesion:** 0.15 - loosely connected
**Members:** 13 nodes

## Members

- [[auth_magic.rs]] - code - api_rust/tests/auth_magic.rs
- [[location_has_error()_2]] - code - api_rust/tests/auth_magic.rs
- [[magic_generate_empty_email_returns_400()]] - code - api_rust/tests/auth_magic.rs
- [[magic_generate_rejects_malformed_emails()]] - code - api_rust/tests/auth_magic.rs
- [[magic_generate_valid_email_returns_key()]] - code - api_rust/tests/auth_magic.rs
- [[magic_sign_in_expired_code_redirects_with_error()]] - code - api_rust/tests/auth_magic.rs
- [[magic_sign_in_unknown_user_redirects_with_error()]] - code - api_rust/tests/auth_magic.rs
- [[magic_sign_in_without_code_redirects_with_error()]] - code - api_rust/tests/auth_magic.rs
- [[magic_sign_in_wrong_code_redirects_with_error()]] - code - api_rust/tests/auth_magic.rs
- [[magic_sign_up_existing_user_redirects_with_error()]] - code - api_rust/tests/auth_magic.rs
- [[magic_sign_up_expired_code_redirects_with_error()]] - code - api_rust/tests/auth_magic.rs
- [[magic_sign_up_without_code_redirects_with_error()]] - code - api_rust/tests/auth_magic.rs
- [[magic_sign_up_wrong_code_redirects_with_error()]] - code - api_rust/tests/auth_magic.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Magic_Sign
SORT file.name ASC
```

## Connections to other communities

- 11 edges to [[_COMMUNITY_Returns Sign]]
- 1 edge to [[_COMMUNITY_Password Email]]

## Top bridge nodes

- [[magic_generate_rejects_malformed_emails()]] - degree 3, connects to 2 communities
- [[magic_generate_empty_email_returns_400()]] - degree 2, connects to 1 community
- [[magic_generate_valid_email_returns_key()]] - degree 2, connects to 1 community
- [[magic_sign_in_expired_code_redirects_with_error()]] - degree 2, connects to 1 community
- [[magic_sign_in_unknown_user_redirects_with_error()]] - degree 2, connects to 1 community
