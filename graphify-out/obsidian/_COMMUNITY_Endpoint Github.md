---
type: community
cohesion: 0.21
members: 15
---

# Endpoint Github

**Cohesion:** 0.21 - loosely connected
**Members:** 15 nodes

## Members

- [[.__get_email()]] - code - api/plane/authentication/provider/oauth/github.py
- [[.get()_48]] - code - api/plane/authentication/views/app/github.py
- [[.get()_47]] - code - api/plane/authentication/views/app/github.py
- [[.get()_55]] - code - api/plane/authentication/views/space/github.py
- [[.is_user_in_organization()]] - code - api/plane/authentication/provider/oauth/github.py
- [[.set_token_data()_3]] - code - api/plane/authentication/provider/oauth/github.py
- [[.set_user_data()_6]] - code - api/plane/authentication/provider/oauth/github.py
- [[GitHubCallbackEndpoint]] - code - api/plane/authentication/views/app/github.py
- [[GitHubCallbackSpaceEndpoint]] - code - api/plane/authentication/views/space/github.py
- [[GitHubOAuthProvider]] - code - api/plane/authentication/provider/oauth/github.py
- [[GitHubOauthInitiateEndpoint]] - code - api/plane/authentication/views/app/github.py
- [[GitHubOauthInitiateSpaceEndpoint]] - code - api/plane/authentication/views/space/github.py
- [[github.py_4]] - code - api/plane/authentication/provider/oauth/github.py
- [[github.py_2]] - code - api/plane/authentication/views/app/github.py
- [[github.py_3]] - code - api/plane/authentication/views/space/github.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Endpoint_Github
SORT file.name ASC
```

## Connections to other communities

- 11 edges to [[_COMMUNITY_Endpoint Magic]]
- 8 edges to [[_COMMUNITY_Path Redirection]]
- 6 edges to [[_COMMUNITY_Endpoint Sign]]
- 1 edge to [[_COMMUNITY_Test Validate]]
- 1 edge to [[_COMMUNITY_User Oauth]]
- 1 edge to [[_COMMUNITY_Google Endpoint]]

## Top bridge nodes

- [[GitHubOAuthProvider]] - degree 17, connects to 4 communities
- [[GitHubCallbackSpaceEndpoint]] - degree 5, connects to 3 communities
- [[.get()_48]] - degree 7, connects to 2 communities
- [[GitHubCallbackEndpoint]] - degree 5, connects to 2 communities
- [[GitHubOauthInitiateEndpoint]] - degree 5, connects to 2 communities
