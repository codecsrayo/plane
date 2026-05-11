---
type: community
cohesion: 0.33
members: 11
---

# Search Returns

**Cohesion:** 0.33 - loosely connected
**Members:** 11 nodes

## Members

- [[entity_search_member_returns_200()]] - code - api_rust/tests/search_tests.rs
- [[entity_search_unauthenticated_returns_401()]] - code - api_rust/tests/search_tests.rs
- [[global_search_empty_query_returns_200()]] - code - api_rust/tests/search_tests.rs
- [[global_search_nonmember_returns_403()]] - code - api_rust/tests/search_tests.rs
- [[global_search_unauthenticated_returns_401()]] - code - api_rust/tests/search_tests.rs
- [[global_search_with_query_returns_200()]] - code - api_rust/tests/search_tests.rs
- [[search_issues_empty_query_returns_200()]] - code - api_rust/tests/search_tests.rs
- [[search_issues_member_returns_200()]] - code - api_rust/tests/search_tests.rs
- [[search_issues_unauthenticated_returns_401()]] - code - api_rust/tests/search_tests.rs
- [[search_tests.rs]] - code - api_rust/tests/search_tests.rs
- [[setup()_32]] - code - api_rust/tests/search_tests.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Search_Returns
SORT file.name ASC
```

## Connections to other communities

- 2 edges to [[_COMMUNITY_Returns Sign]]

## Top bridge nodes

- [[setup()_32]] - degree 10, connects to 1 community
- [[global_search_unauthenticated_returns_401()]] - degree 2, connects to 1 community
