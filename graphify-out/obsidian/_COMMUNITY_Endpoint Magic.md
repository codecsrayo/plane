---
type: community
cohesion: 0.09
members: 45
---

# Endpoint Magic

**Cohesion:** 0.09 - loosely connected
**Members:** 45 nodes

## Members

- [[.__init__()_29]] - code - api/plane/authentication/adapter/error.py
- [[.__init__()_30]] - code - api/plane/authentication/provider/credentials/magic_code.py
- [[.__init__()_34]] - code - api/plane/authentication/provider/oauth/github.py
- [[.__init__()_32]] - code - api/plane/authentication/provider/oauth/google.py
- [[.get_error_dict()]] - code - api/plane/authentication/adapter/error.py
- [[.initiate()]] - code - api/plane/authentication/provider/credentials/magic_code.py
- [[.post()_21]] - code - api/plane/authentication/views/app/check.py
- [[.post()_22]] - code - api/plane/authentication/views/app/magic.py
- [[.post()_19]] - code - api/plane/authentication/views/app/password_management.py
- [[.post()_20]] - code - api/plane/authentication/views/app/password_management.py
- [[.post()_30]] - code - api/plane/authentication/views/space/check.py
- [[.post()_31]] - code - api/plane/authentication/views/space/magic.py
- [[.post()_28]] - code - api/plane/authentication/views/space/password_management.py
- [[.post()_29]] - code - api/plane/authentication/views/space/password_management.py
- [[.set_user_data()_2]] - code - api/plane/authentication/provider/credentials/magic_code.py
- [[.throttle_failure_view()]] - code - api/plane/authentication/rate_limit.py
- [[APIView]] - code
- [[AnonRateThrottle]] - code
- [[AuthenticationException]] - code - api/plane/authentication/adapter/error.py
- [[AuthenticationThrottle]] - code - api/plane/authentication/rate_limit.py
- [[EmailCheckEndpoint]] - code - api/plane/authentication/views/app/check.py
- [[EmailCheckSpaceEndpoint]] - code - api/plane/authentication/views/space/check.py
- [[ForgotPasswordEndpoint]] - code - api/plane/authentication/views/app/password_management.py
- [[ForgotPasswordSpaceEndpoint]] - code - api/plane/authentication/views/space/password_management.py
- [[MagicCodeProvider]] - code - api/plane/authentication/provider/credentials/magic_code.py
- [[MagicGenerateEndpoint]] - code - api/plane/authentication/views/app/magic.py
- [[MagicGenerateSpaceEndpoint]] - code - api/plane/authentication/views/space/magic.py
- [[MagicSignInEndpoint]] - code - api/plane/authentication/views/app/magic.py
- [[MagicSignInSpaceEndpoint]] - code - api/plane/authentication/views/space/magic.py
- [[MagicSignUpEndpoint]] - code - api/plane/authentication/views/app/magic.py
- [[MagicSignUpSpaceEndpoint]] - code - api/plane/authentication/views/space/magic.py
- [[ResetPasswordEndpoint]] - code - api/plane/authentication/views/app/password_management.py
- [[ResetPasswordSpaceEndpoint]] - code - api/plane/authentication/views/space/password_management.py
- [[SetUserPasswordEndpoint]] - code - api/plane/authentication/views/common.py
- [[TimezoneEndpoint]] - code - api/plane/app/views/timezone/base.py
- [[check.py]] - code - api/plane/authentication/views/app/check.py
- [[check.py_1]] - code - api/plane/authentication/views/space/check.py
- [[generate_password_token()]] - code - api/plane/authentication/views/app/password_management.py
- [[generate_password_token()_1]] - code - api/plane/authentication/views/space/password_management.py
- [[get_configuration_value()]] - code - api/plane/license/utils/instance_value.py
- [[magic.py]] - code - api/plane/authentication/views/app/magic.py
- [[magic.py_1]] - code - api/plane/authentication/views/space/magic.py
- [[magic_code.py]] - code - api/plane/authentication/provider/credentials/magic_code.py
- [[password_management.py]] - code - api/plane/authentication/views/app/password_management.py
- [[password_management.py_1]] - code - api/plane/authentication/views/space/password_management.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Endpoint_Magic
SORT file.name ASC
```

## Connections to other communities

- 35 edges to [[_COMMUNITY_Endpoint Sign]]
- 33 edges to [[_COMMUNITY_Path Redirection]]
- 11 edges to [[_COMMUNITY_Endpoint Github]]
- 9 edges to [[_COMMUNITY_Gitea Endpoint]]
- 8 edges to [[_COMMUNITY_Partial Endpoint]]
- 7 edges to [[_COMMUNITY_Endpoint Common]]
- 7 edges to [[_COMMUNITY_Google Endpoint]]
- 6 edges to [[_COMMUNITY_Endpoint User]]
- 6 edges to [[_COMMUNITY_Check User]]
- 4 edges to [[_COMMUNITY_User Oauth]]
- 2 edges to [[_COMMUNITY_Provider Endpoint]]
- 2 edges to [[_COMMUNITY_Test Magic]]
- 2 edges to [[_COMMUNITY_Email Task]]
- 1 edge to [[_COMMUNITY_Endpoint Workspace]]
- 1 edge to [[_COMMUNITY_Event Tracking]]
- 1 edge to [[_COMMUNITY_Endpoint User]]
- 1 edge to [[_COMMUNITY_Installation Access]]
- 1 edge to [[_COMMUNITY_Test Validate]]
- 1 edge to [[_COMMUNITY_Instance Endpoint]]
- 1 edge to [[_COMMUNITY_Task Object]]
- 1 edge to [[_COMMUNITY_Command Issue]]

## Top bridge nodes

- [[AuthenticationException]] - degree 110, connects to 12 communities
- [[get_configuration_value()]] - degree 24, connects to 11 communities
- [[MagicCodeProvider]] - degree 18, connects to 2 communities
- [[APIView]] - degree 11, connects to 2 communities
- [[MagicSignInEndpoint]] - degree 6, connects to 2 communities
