---
type: community
cohesion: 0.15
members: 13
---

# Space Sign

**Cohesion:** 0.15 - loosely connected
**Members:** 13 nodes

## Members

- [[auth_spaces.rs]] - code - api_rust/tests/auth_spaces.rs
- [[location_has_error()_3]] - code - api_rust/tests/auth_spaces.rs
- [[space_email_check_accepts_valid_email()]] - code - api_rust/tests/auth_spaces.rs
- [[space_email_check_rejects_empty_email()]] - code - api_rust/tests/auth_spaces.rs
- [[space_forgot_password_without_smtp_returns_400()]] - code - api_rust/tests/auth_spaces.rs
- [[space_magic_generate_invalid_email_returns_400()]] - code - api_rust/tests/auth_spaces.rs
- [[space_magic_generate_valid_email_returns_key()]] - code - api_rust/tests/auth_spaces.rs
- [[space_magic_sign_in_without_code_redirects_to_space_base()]] - code - api_rust/tests/auth_spaces.rs
- [[space_magic_sign_up_without_code_redirects_to_space_base()]] - code - api_rust/tests/auth_spaces.rs
- [[space_reset_password_invalid_uidb64_redirects_to_space_base()]] - code - api_rust/tests/auth_spaces.rs
- [[space_sign_in_missing_credentials_redirects_to_space_base()]] - code - api_rust/tests/auth_spaces.rs
- [[space_sign_in_unknown_user_redirects_with_error()]] - code - api_rust/tests/auth_spaces.rs
- [[space_sign_out_without_session_returns_401()]] - code - api_rust/tests/auth_spaces.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Space_Sign
SORT file.name ASC
```

## Connections to other communities

- 11 edges to [[_COMMUNITY_Returns Sign]]

## Top bridge nodes

- [[space_email_check_accepts_valid_email()]] - degree 2, connects to 1 community
- [[space_email_check_rejects_empty_email()]] - degree 2, connects to 1 community
- [[space_forgot_password_without_smtp_returns_400()]] - degree 2, connects to 1 community
- [[space_magic_generate_invalid_email_returns_400()]] - degree 2, connects to 1 community
- [[space_magic_generate_valid_email_returns_key()]] - degree 2, connects to 1 community
