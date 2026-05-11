---
type: community
cohesion: 0.09
members: 47
---

# Auth Rust API

**Cohesion:** 0.09 - loosely connected
**Members:** 47 nodes

## Members

- [[.email_required()]] - code - api_rust/src/auth/responses.rs
- [[.expired_password_token()]] - code - api_rust/src/auth/responses.rs
- [[.incorrect_old_password()]] - code - api_rust/src/auth/responses.rs
- [[.instance_not_configured()]] - code - api_rust/src/auth/responses.rs
- [[.into_response()_1]] - code - api_rust/src/auth/responses.rs
- [[.invalid_email()]] - code - api_rust/src/auth/responses.rs
- [[.invalid_password()]] - code - api_rust/src/auth/responses.rs
- [[.invalid_password_token()]] - code - api_rust/src/auth/responses.rs
- [[.missing_password()]] - code - api_rust/src/auth/responses.rs
- [[.new()]] - code - api_rust/src/auth/responses.rs
- [[.password_already_set()]] - code - api_rust/src/auth/responses.rs
- [[.password_too_weak()]] - code - api_rust/src/auth/responses.rs
- [[.smtp_not_configured()]] - code - api_rust/src/auth/responses.rs
- [[.user_does_not_exist()]] - code - api_rust/src/auth/responses.rs
- [[AuthError]] - code - api_rust/src/auth/responses.rs
- [[ChangePasswordRequest]] - code - api_rust/src/auth/password_management.rs
- [[EmailCheckRequest]] - code - api_rust/src/auth/email_check.rs
- [[ForgotPasswordRequest]] - code - api_rust/src/auth/forgot_reset_password.rs
- [[ResetPasswordForm]] - code - api_rust/src/auth/forgot_reset_password.rs
- [[ResetTokenData]] - code - api_rust/src/auth/forgot_reset_password.rs
- [[SetPasswordRequest]] - code - api_rust/src/auth/password_management.rs
- [[WorkspaceSeedJob]] - code - api_rust/src/jobs/workspace_seed.rs
- [[builder()]] - code - api_rust/src/utils/content_validator.rs
- [[change_password()]] - code - api_rust/src/auth/password_management.rs
- [[decode_uidb64()]] - code - api_rust/src/auth/forgot_reset_password.rs
- [[email_check()]] - code - api_rust/src/auth/email_check.rs
- [[email_check.rs]] - code - api_rust/src/auth/email_check.rs
- [[email_check_space()]] - code - api_rust/src/auth/email_check.rs
- [[ensure_magic_enabled()]] - code - api_rust/src/auth/magic_auth.rs
- [[forgot_password()]] - code - api_rust/src/auth/forgot_reset_password.rs
- [[forgot_password_space()]] - code - api_rust/src/auth/forgot_reset_password.rs
- [[forgot_reset_password.rs]] - code - api_rust/src/auth/forgot_reset_password.rs
- [[generate_magic_code()]] - code - api_rust/src/auth/magic_auth.rs
- [[handle_forgot_password()]] - code - api_rust/src/auth/forgot_reset_password.rs
- [[handle_reset_password()]] - code - api_rust/src/auth/forgot_reset_password.rs
- [[handle_workspace_seed()]] - code - api_rust/src/jobs/workspace_seed.rs
- [[password_management.rs]] - code - api_rust/src/auth/password_management.rs
- [[reset_password()]] - code - api_rust/src/auth/forgot_reset_password.rs
- [[reset_password_space()]] - code - api_rust/src/auth/forgot_reset_password.rs
- [[run_email_check()]] - code - api_rust/src/auth/email_check.rs
- [[run_seed()]] - code - api_rust/src/jobs/workspace_seed.rs
- [[send_magic_code_email()]] - code - api_rust/src/auth/magic_auth.rs
- [[send_reset_email()]] - code - api_rust/src/auth/forgot_reset_password.rs
- [[set_password()]] - code - api_rust/src/auth/password_management.rs
- [[validate_csrf_header()]] - code - api_rust/src/auth/password_management.rs
- [[validate_password_strength()]] - code - api_rust/src/auth/password_management.rs
- [[workspace_seed.rs]] - code - api_rust/src/jobs/workspace_seed.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Auth_Rust_API
SORT file.name ASC
```

## Connections to other communities

- 10 edges to [[_COMMUNITY_Community 50]]
- 10 edges to [[_COMMUNITY_Community 40]]
- 8 edges to [[_COMMUNITY_Community 43]]
- 7 edges to [[_COMMUNITY_Community 73]]
- 7 edges to [[_COMMUNITY_Community 135]]
- 6 edges to [[_COMMUNITY_Community 57]]
- 6 edges to [[_COMMUNITY_Community 136]]
- 6 edges to [[_COMMUNITY_Workspace Activity & Analytics]]
- 6 edges to [[_COMMUNITY_Community 187]]
- 5 edges to [[_COMMUNITY_Community 144]]
- 5 edges to [[_COMMUNITY_Cycles Rust API]]
- 5 edges to [[_COMMUNITY_Community 30]]
- 5 edges to [[_COMMUNITY_Issue Routes Rust API]]
- 5 edges to [[_COMMUNITY_Projects Rust API]]
- 4 edges to [[_COMMUNITY_Community 129]]
- 4 edges to [[_COMMUNITY_Modules Rust API]]
- 4 edges to [[_COMMUNITY_Workspace & Issues Rust API]]
- 4 edges to [[_COMMUNITY_Community 171]]
- 4 edges to [[_COMMUNITY_Community 107]]
- 4 edges to [[_COMMUNITY_Community 229]]
- 4 edges to [[_COMMUNITY_Community 181]]
- 3 edges to [[_COMMUNITY_Community 156]]
- 3 edges to [[_COMMUNITY_Community 108]]
- 3 edges to [[_COMMUNITY_Community 34]]
- 3 edges to [[_COMMUNITY_Community 114]]
- 2 edges to [[_COMMUNITY_Community 91]]
- 2 edges to [[_COMMUNITY_Community 143]]
- 2 edges to [[_COMMUNITY_Community 280]]
- 1 edge to [[_COMMUNITY_Community 170]]
- 1 edge to [[_COMMUNITY_Community 295]]
- 1 edge to [[_COMMUNITY_Community 195]]
- 1 edge to [[_COMMUNITY_Community 207]]
- 1 edge to [[_COMMUNITY_Community 238]]
- 1 edge to [[_COMMUNITY_Community 130]]
- 1 edge to [[_COMMUNITY_Community 51]]
- 1 edge to [[_COMMUNITY_Community 66]]
- 1 edge to [[_COMMUNITY_Community 47]]
- 1 edge to [[_COMMUNITY_Community 116]]
- 1 edge to [[_COMMUNITY_Community 115]]
- 1 edge to [[_COMMUNITY_Community 180]]
- 1 edge to [[_COMMUNITY_Community 228]]
- 1 edge to [[_COMMUNITY_Community 157]]

## Top bridge nodes

- [[.new()]] - degree 151, connects to 41 communities
- [[builder()]] - degree 10, connects to 5 communities
- [[ensure_magic_enabled()]] - degree 6, connects to 2 communities
- [[send_magic_code_email()]] - degree 5, connects to 2 communities
- [[AuthError]] - degree 15, connects to 1 community
