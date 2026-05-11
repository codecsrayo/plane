---
type: community
cohesion: 0.21
members: 13
---

# Gitea Endpoint

**Cohesion:** 0.21 - loosely connected
**Members:** 13 nodes

## Members

- [[.__get_email()_1]] - code - api/plane/authentication/provider/oauth/gitea.py
- [[.__init__()_35]] - code - api/plane/authentication/provider/oauth/gitea.py
- [[.get()_49]] - code - api/plane/authentication/views/app/gitea.py
- [[.set_token_data()_4]] - code - api/plane/authentication/provider/oauth/gitea.py
- [[.set_user_data()_7]] - code - api/plane/authentication/provider/oauth/gitea.py
- [[GiteaCallbackEndpoint]] - code - api/plane/authentication/views/app/gitea.py
- [[GiteaCallbackSpaceEndpoint]] - code - api/plane/authentication/views/space/gitea.py
- [[GiteaOAuthProvider]] - code - api/plane/authentication/provider/oauth/gitea.py
- [[GiteaOauthInitiateEndpoint]] - code - api/plane/authentication/views/app/gitea.py
- [[GiteaOauthInitiateSpaceEndpoint]] - code - api/plane/authentication/views/space/gitea.py
- [[gitea.py_2]] - code - api/plane/authentication/provider/oauth/gitea.py
- [[gitea.py]] - code - api/plane/authentication/views/app/gitea.py
- [[gitea.py_1]] - code - api/plane/authentication/views/space/gitea.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Gitea_Endpoint
SORT file.name ASC
```

## Connections to other communities

- 9 edges to [[_COMMUNITY_Endpoint Magic]]
- 8 edges to [[_COMMUNITY_Path Redirection]]
- 6 edges to [[_COMMUNITY_Endpoint Sign]]
- 1 edge to [[_COMMUNITY_User Oauth]]
- 1 edge to [[_COMMUNITY_Google Endpoint]]

## Top bridge nodes

- [[GiteaOAuthProvider]] - degree 16, connects to 4 communities
- [[GiteaCallbackEndpoint]] - degree 5, connects to 3 communities
- [[GiteaCallbackSpaceEndpoint]] - degree 5, connects to 3 communities
- [[GiteaOauthInitiateSpaceEndpoint]] - degree 5, connects to 3 communities
- [[GiteaOauthInitiateEndpoint]] - degree 5, connects to 2 communities
