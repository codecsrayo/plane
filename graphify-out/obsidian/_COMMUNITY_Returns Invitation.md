---
type: community
cohesion: 0.27
members: 13
---

# Returns Invitation

**Cohesion:** 0.27 - loosely connected
**Members:** 13 nodes

## Members

- [[create_invitation_as_admin_returns_2xx()]] - code - api_rust/tests/workspace_invitations.rs
- [[create_invitation_unauthenticated_returns_401()]] - code - api_rust/tests/workspace_invitations.rs
- [[delete_invitation_nonexistent_returns_4xx()]] - code - api_rust/tests/workspace_invitations.rs
- [[delete_invitation_unauthenticated_returns_401()]] - code - api_rust/tests/workspace_invitations.rs
- [[get_invitation_nonexistent_returns_404_or_403()]] - code - api_rust/tests/workspace_invitations.rs
- [[join_invitation_with_invalid_token_returns_4xx()]] - code - api_rust/tests/workspace_invitations.rs
- [[list_invitations_member_returns_200()]] - code - api_rust/tests/workspace_invitations.rs
- [[list_invitations_unauthenticated_returns_401()]] - code - api_rust/tests/workspace_invitations.rs
- [[list_user_workspace_invitations_returns_200()]] - code - api_rust/tests/workspace_invitations.rs
- [[list_user_workspace_invitations_unauthenticated_returns_401()]] - code - api_rust/tests/workspace_invitations.rs
- [[setup()_15]] - code - api_rust/tests/workspace_invitations.rs
- [[update_invitation_nonexistent_returns_4xx()]] - code - api_rust/tests/workspace_invitations.rs
- [[workspace_invitations.rs]] - code - api_rust/tests/workspace_invitations.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Returns_Invitation
SORT file.name ASC
```

## Connections to other communities

- 3 edges to [[_COMMUNITY_Returns Sign]]

## Top bridge nodes

- [[setup()_15]] - degree 11, connects to 1 community
- [[list_user_workspace_invitations_returns_200()]] - degree 2, connects to 1 community
- [[list_user_workspace_invitations_unauthenticated_returns_401()]] - degree 2, connects to 1 community
