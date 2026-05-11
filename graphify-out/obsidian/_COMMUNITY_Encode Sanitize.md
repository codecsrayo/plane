---
type: community
cohesion: 0.12
members: 25
---

# Encode Sanitize

**Cohesion:** 0.12 - loosely connected
**Members:** 25 nodes

## Members

- [[ExportIssuesJob]] - code - api_rust/src/jobs/export.rs
- [[IssueRow]] - code - api_rust/src/jobs/export.rs
- [[RelationMaps]] - code - api_rust/src/jobs/export.rs
- [[build_rows()]] - code - api_rust/src/jobs/export.rs
- [[build_zip()]] - code - api_rust/src/jobs/export.rs
- [[csv_sanitize.rs]] - code - api_rust/src/utils/csv_sanitize.rs
- [[empty_string_passes_through()]] - code - api_rust/src/utils/csv_sanitize.rs
- [[encode_csv()]] - code - api_rust/src/jobs/export.rs
- [[encode_issues()]] - code - api_rust/src/jobs/export.rs
- [[encode_json()]] - code - api_rust/src/jobs/export.rs
- [[encode_xlsx()]] - code - api_rust/src/jobs/export.rs
- [[export.rs]] - code - api_rust/src/jobs/export.rs
- [[fetch_related_maps()]] - code - api_rust/src/jobs/export.rs
- [[format_user_name()]] - code - api_rust/src/jobs/export.rs
- [[handle_export_issues()]] - code - api_rust/src/jobs/export.rs
- [[mark_export_failed()]] - code - api_rust/src/jobs/export.rs
- [[pass_through_when_safe()]] - code - api_rust/src/utils/csv_sanitize.rs
- [[prefixes_formula_triggers()]] - code - api_rust/src/utils/csv_sanitize.rs
- [[prefixes_whitespace_triggers()]] - code - api_rust/src/utils/csv_sanitize.rs
- [[project_label()]] - code - api_rust/src/jobs/export.rs
- [[run_export()]] - code - api_rust/src/jobs/export.rs
- [[sanitize_csv_cell()]] - code - api_rust/src/utils/csv_sanitize.rs
- [[sanitize_filename_segment()]] - code - api_rust/src/jobs/export.rs
- [[unicode_first_char_is_safe()]] - code - api_rust/src/utils/csv_sanitize.rs
- [[unique_base_name()]] - code - api_rust/src/jobs/export.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Encode_Sanitize
SORT file.name ASC
```

## Connections to other communities

- 7 edges to [[_COMMUNITY_Password Email]]
- 4 edges to [[_COMMUNITY_Asset Issue]]
- 2 edges to [[_COMMUNITY_Issue Request]]
- 1 edge to [[_COMMUNITY_Workspace User]]

## Top bridge nodes

- [[run_export()]] - degree 12, connects to 3 communities
- [[mark_export_failed()]] - degree 4, connects to 2 communities
- [[encode_csv()]] - degree 4, connects to 1 community
- [[fetch_related_maps()]] - degree 4, connects to 1 community
- [[handle_export_issues()]] - degree 4, connects to 1 community
