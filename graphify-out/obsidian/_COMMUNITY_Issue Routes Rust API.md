---
type: community
cohesion: 0.05
members: 75
---

# Issue Routes Rust API

**Cohesion:** 0.05 - loosely connected
**Members:** 75 nodes

## Members

- [[.empty()]] - code - api_rust/src/routes/issue_pagination.rs
- [[.from()_2]] - code - api_rust/src/routes/issue_description_versions.rs
- [[.from()_1]] - code - api_rust/src/routes/issue_description_versions.rs
- [[.to_filter_params()_1]] - code - api_rust/src/routes/issues.rs
- [[.to_filter_params()_2]] - code - api_rust/src/routes/user_profile_issues.rs
- [[.to_filter_params()]] - code - api_rust/src/routes/workspace_view_issues.rs
- [[CreateIssueRequest]] - code - api_rust/src/routes/issues.rs
- [[DescriptionVersionDetail]] - code - api_rust/src/routes/issue_description_versions.rs
- [[DescriptionVersionListItem]] - code - api_rust/src/routes/issue_description_versions.rs
- [[DescriptionVersionsQuery]] - code - api_rust/src/routes/issue_description_versions.rs
- [[EnrichmentMaps]] - code - api_rust/src/routes/issue_pagination.rs
- [[FilteredQuery]] - code - api_rust/src/routes/issue_filters.rs
- [[IssueCreateResponse]] - code - api_rust/src/routes/issues.rs
- [[IssueDetailQuery]] - code - api_rust/src/routes/issues.rs
- [[IssueDetailResponse]] - code - api_rust/src/routes/issues.rs
- [[IssueFilterParams]] - code - api_rust/src/routes/issue_filters.rs
- [[IssueListByIdsQuery]] - code - api_rust/src/routes/issues.rs
- [[ListIssuesQuery]] - code - api_rust/src/routes/issues.rs
- [[ProjectIssueItem]] - code - api_rust/src/routes/issues.rs
- [[UpdateIssueRequest]] - code - api_rust/src/routes/issues.rs
- [[UserProfileIssueItem]] - code - api_rust/src/routes/user_profile_issues.rs
- [[UserProfileIssuesQuery]] - code - api_rust/src/routes/user_profile_issues.rs
- [[V2IssueItem]] - code - api_rust/src/routes/issues.rs
- [[V2IssuesQuery]] - code - api_rust/src/routes/issues.rs
- [[WorkspaceIssueItem]] - code - api_rust/src/routes/workspace_view_issues.rs
- [[WorkspaceIssuesQuery]] - code - api_rust/src/routes/workspace_view_issues.rs
- [[apply_cycle_membership()]] - code - api_rust/src/routes/issue_filters.rs
- [[apply_date_filter()]] - code - api_rust/src/routes/issue_filters.rs
- [[apply_issue_filters()]] - code - api_rust/src/routes/issue_filters.rs
- [[apply_issue_order()]] - code - api_rust/src/routes/issue_pagination.rs
- [[apply_module_membership()]] - code - api_rust/src/routes/issue_filters.rs
- [[apply_nullable_uuid_filter()]] - code - api_rust/src/routes/issue_filters.rs
- [[build_create_response()]] - code - api_rust/src/routes/issues.rs
- [[build_detail_response()]] - code - api_rust/src/routes/issues.rs
- [[check_guest_issue_access()]] - code - api_rust/src/routes/issue_description_versions.rs
- [[collect_state_ids()]] - code - api_rust/src/routes/issue_pagination.rs
- [[create_issue()]] - code - api_rust/src/routes/issues.rs
- [[csv_contains_none()]] - code - api_rust/src/routes/issue_filters.rs
- [[dedup()]] - code - api_rust/src/routes/issue_filters.rs
- [[delete_issue()]] - code - api_rust/src/routes/issues.rs
- [[empty_paginated_response()]] - code - api_rust/src/routes/issue_pagination.rs
- [[get_description_version()]] - code - api_rust/src/routes/issue_description_versions.rs
- [[get_issue()]] - code - api_rust/src/routes/issues.rs
- [[issue_description_versions.rs]] - code - api_rust/src/routes/issue_description_versions.rs
- [[issue_filters.rs]] - code - api_rust/src/routes/issue_filters.rs
- [[issue_pagination.rs]] - code - api_rust/src/routes/issue_pagination.rs
- [[issues.rs]] - code - api_rust/src/routes/issues.rs
- [[json_filter_value_to_csv()]] - code - api_rust/src/routes/issue_filters.rs
- [[list_description_versions()]] - code - api_rust/src/routes/issue_description_versions.rs
- [[list_issues()]] - code - api_rust/src/routes/issues.rs
- [[list_issues_by_ids()]] - code - api_rust/src/routes/issues.rs
- [[list_issues_detail()]] - code - api_rust/src/routes/issues.rs
- [[list_issues_v2()]] - code - api_rust/src/routes/issues.rs
- [[list_user_profile_issues()]] - code - api_rust/src/routes/user_profile_issues.rs
- [[list_workspace_view_issues()]] - code - api_rust/src/routes/workspace_view_issues.rs
- [[load_enrichment()]] - code - api_rust/src/routes/issue_pagination.rs
- [[load_issues_in_cycles()]] - code - api_rust/src/routes/issue_filters.rs
- [[load_issues_in_modules()]] - code - api_rust/src/routes/issue_filters.rs
- [[load_issues_with_assignees()]] - code - api_rust/src/routes/issue_filters.rs
- [[load_issues_with_labels()]] - code - api_rust/src/routes/issue_filters.rs
- [[load_issues_with_subscribers()]] - code - api_rust/src/routes/issue_filters.rs
- [[load_state_ids_by_groups()]] - code - api_rust/src/routes/issue_filters.rs
- [[load_target_user_issue_ids()]] - code - api_rust/src/routes/user_profile_issues.rs
- [[load_triage_state_ids()]] - code - api_rust/src/routes/issue_pagination.rs
- [[load_workspace_triage_state_ids()]] - code - api_rust/src/routes/issue_pagination.rs
- [[merge_json_filters()]] - code - api_rust/src/routes/issue_filters.rs
- [[paginated_response()]] - code - api_rust/src/routes/issue_pagination.rs
- [[parse_cursor()]] - code - api_rust/src/routes/issue_pagination.rs
- [[parse_uuids_csv()]] - code - api_rust/src/routes/issue_filters.rs
- [[split_csv()]] - code - api_rust/src/routes/issue_filters.rs
- [[sync_assignees()]] - code - api_rust/src/routes/issues.rs
- [[sync_labels()]] - code - api_rust/src/routes/issues.rs
- [[update_issue()]] - code - api_rust/src/routes/issues.rs
- [[user_profile_issues.rs]] - code - api_rust/src/routes/user_profile_issues.rs
- [[workspace_view_issues.rs]] - code - api_rust/src/routes/workspace_view_issues.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Issue_Routes_Rust_API
SORT file.name ASC
```

## Connections to other communities

- 10 edges to [[_COMMUNITY_Modules Rust API]]
- 5 edges to [[_COMMUNITY_Auth Rust API]]
- 2 edges to [[_COMMUNITY_Community 171]]
- 1 edge to [[_COMMUNITY_Workspace & Issues Rust API]]
- 1 edge to [[_COMMUNITY_Community 40]]

## Top bridge nodes

- [[list_user_profile_issues()]] - degree 14, connects to 2 communities
- [[create_issue()]] - degree 6, connects to 2 communities
- [[update_issue()]] - degree 6, connects to 2 communities
- [[list_issues_by_ids()]] - degree 5, connects to 2 communities
- [[list_issues()]] - degree 12, connects to 1 community
