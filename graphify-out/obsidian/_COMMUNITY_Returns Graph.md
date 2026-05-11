---
type: community
cohesion: 0.21
members: 13
---

# Returns Graph

**Cohesion:** 0.21 - loosely connected
**Members:** 13 nodes

## Members

- [[get_activity_graph_returns_200()]] - code - api_rust/tests/users_extended.rs
- [[get_issues_completed_graph_returns_200()]] - code - api_rust/tests/users_extended.rs
- [[get_last_workspace_returns_200_or_404()]] - code - api_rust/tests/users_extended.rs
- [[get_my_activities_returns_200()]] - code - api_rust/tests/users_extended.rs
- [[get_profile_returns_200()]] - code - api_rust/tests/users_extended.rs
- [[get_workspace_dashboard_returns_200()]] - code - api_rust/tests/users_extended.rs
- [[list_accounts_returns_200()]] - code - api_rust/tests/users_extended.rs
- [[list_user_workspaces_returns_200()]] - code - api_rust/tests/users_extended.rs
- [[seed_profile()]] - code - api_rust/tests/users_extended.rs
- [[update_onboard_returns_200()]] - code - api_rust/tests/users_extended.rs
- [[update_profile_returns_200()]] - code - api_rust/tests/users_extended.rs
- [[update_tour_completed_returns_200()]] - code - api_rust/tests/users_extended.rs
- [[users_extended.rs]] - code - api_rust/tests/users_extended.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Returns_Graph
SORT file.name ASC
```

## Connections to other communities

- 11 edges to [[_COMMUNITY_Returns Sign]]
- 1 edge to [[_COMMUNITY_Issue Request]]
- 1 edge to [[_COMMUNITY_Asset Issue]]
- 1 edge to [[_COMMUNITY_Password Email]]

## Top bridge nodes

- [[seed_profile()]] - degree 8, connects to 3 communities
- [[get_profile_returns_200()]] - degree 3, connects to 1 community
- [[update_onboard_returns_200()]] - degree 3, connects to 1 community
- [[update_profile_returns_200()]] - degree 3, connects to 1 community
- [[update_tour_completed_returns_200()]] - degree 3, connects to 1 community
