---
type: community
cohesion: 0.13
members: 23
---

# Check User

**Cohesion:** 0.13 - loosely connected
**Members:** 23 nodes

## Members

- [[.__check_signup()]] - code - api/plane/authentication/adapter/base.py
- [[.__init__()_27]] - code - api/plane/authentication/adapter/base.py
- [[.authenticate()_1]] - code - api/plane/authentication/adapter/base.py
- [[.check_sync_enabled()]] - code - api/plane/authentication/adapter/base.py
- [[.complete_login_or_signup()]] - code - api/plane/authentication/adapter/base.py
- [[.create_update_account()]] - code - api/plane/authentication/adapter/base.py
- [[.delete_old_avatar()]] - code - api/plane/authentication/adapter/base.py
- [[.download_and_upload_avatar()]] - code - api/plane/authentication/adapter/base.py
- [[.get_avatar_download_headers()]] - code - api/plane/authentication/adapter/base.py
- [[.get_user_response()]] - code - api/plane/authentication/adapter/base.py
- [[.get_user_token()]] - code - api/plane/authentication/adapter/base.py
- [[.sanitize_email()]] - code - api/plane/authentication/adapter/base.py
- [[.set_token_data()]] - code - api/plane/authentication/adapter/base.py
- [[.set_user_data()]] - code - api/plane/authentication/adapter/base.py
- [[.sync_user_data()]] - code - api/plane/authentication/adapter/base.py
- [[.validate_password()]] - code - api/plane/authentication/adapter/base.py
- [[Adapter]] - code - api/plane/authentication/adapter/base.py
- [[Check if sign up is enabled or not and raise exception if not enabled]] - rationale - api/plane/authentication/adapter/base.py
- [[Check if sync is enabled for the provider]] - rationale - api/plane/authentication/adapter/base.py
- [[Common interface for all auth providers]] - rationale - api/plane/authentication/adapter/base.py
- [[Delete the old avatar if it exists]] - rationale - api/plane/authentication/adapter/base.py
- [[Downloads avatar from OAuth provider and uploads to our storage.         Returns]] - rationale - api/plane/authentication/adapter/base.py
- [[Validate password strength]] - rationale - api/plane/authentication/adapter/base.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Check_User
SORT file.name ASC
```

## Connections to other communities

- 6 edges to [[_COMMUNITY_Endpoint Magic]]
- 3 edges to [[_COMMUNITY_Endpoint Issue]]
- 2 edges to [[_COMMUNITY_Task Object]]
- 2 edges to [[_COMMUNITY_User Oauth]]
- 2 edges to [[_COMMUNITY_Middleware Logger]]
- 1 edge to [[_COMMUNITY_Path Redirection]]

## Top bridge nodes

- [[Adapter]] - degree 22, connects to 4 communities
- [[.download_and_upload_avatar()]] - degree 7, connects to 2 communities
- [[.delete_old_avatar()]] - degree 5, connects to 2 communities
- [[.sanitize_email()]] - degree 4, connects to 2 communities
- [[.complete_login_or_signup()]] - degree 9, connects to 1 community
