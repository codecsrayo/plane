---
type: community
cohesion: 0.21
members: 12
---

# Google Endpoint

**Cohesion:** 0.21 - loosely connected
**Members:** 12 nodes

## Members

- [[.get()_43]] - code - api/plane/authentication/views/app/google.py
- [[.set_token_data()_1]] - code - api/plane/authentication/provider/oauth/google.py
- [[.set_user_data()_4]] - code - api/plane/authentication/provider/oauth/google.py
- [[GoogleCallbackEndpoint]] - code - api/plane/authentication/views/app/google.py
- [[GoogleCallbackSpaceEndpoint]] - code - api/plane/authentication/views/space/google.py
- [[GoogleOAuthProvider]] - code - api/plane/authentication/provider/oauth/google.py
- [[GoogleOauthInitiateEndpoint]] - code - api/plane/authentication/views/app/google.py
- [[GoogleOauthInitiateSpaceEndpoint]] - code - api/plane/authentication/views/space/google.py
- [[OauthAdapter_1]] - code
- [[google.py_2]] - code - api/plane/authentication/provider/oauth/google.py
- [[google.py]] - code - api/plane/authentication/views/app/google.py
- [[google.py_1]] - code - api/plane/authentication/views/space/google.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Google_Endpoint
SORT file.name ASC
```

## Connections to other communities

- 8 edges to [[_COMMUNITY_Path Redirection]]
- 7 edges to [[_COMMUNITY_Endpoint Sign]]
- 7 edges to [[_COMMUNITY_Endpoint Magic]]
- 1 edge to [[_COMMUNITY_User Oauth]]
- 1 edge to [[_COMMUNITY_Endpoint Github]]
- 1 edge to [[_COMMUNITY_Gitea Endpoint]]

## Top bridge nodes

- [[GoogleOAuthProvider]] - degree 15, connects to 3 communities
- [[GoogleCallbackEndpoint]] - degree 5, connects to 3 communities
- [[GoogleCallbackSpaceEndpoint]] - degree 5, connects to 3 communities
- [[GoogleOauthInitiateSpaceEndpoint]] - degree 5, connects to 3 communities
- [[OauthAdapter_1]] - degree 4, connects to 3 communities
