---
type: community
cohesion: 0.16
members: 21
---

# Community 115

**Cohesion:** 0.16 - loosely connected
**Members:** 21 nodes

## Members

- [[cleanup.rs]] - code - api_rust/src/jobs/cleanup.rs
- [[cron.rs]] - code - api_rust/src/jobs/cron.rs
- [[daily_at()]] - code - api_rust/src/jobs/cron.rs
- [[delete_api_logs()]] - code - api_rust/src/jobs/cleanup.rs
- [[delete_email_notification_logs()]] - code - api_rust/src/jobs/cleanup.rs
- [[delete_issue_description_versions()]] - code - api_rust/src/jobs/cleanup.rs
- [[delete_old_s3_links()]] - code - api_rust/src/jobs/cleanup.rs
- [[delete_page_versions()]] - code - api_rust/src/jobs/cleanup.rs
- [[delete_unuploaded_file_assets()]] - code - api_rust/src/jobs/cleanup.rs
- [[delete_webhook_logs()]] - code - api_rust/src/jobs/cleanup.rs
- [[enqueue_issue_automation()]] - code - api_rust/src/jobs/cron.rs
- [[find_tables_with_deleted_at()]] - code - api_rust/src/jobs/cleanup.rs
- [[hard_delete()]] - code - api_rust/src/jobs/cleanup.rs
- [[instance_traces()]] - code - api_rust/src/jobs/instance_traces.rs
- [[instance_traces.rs]] - code - api_rust/src/jobs/instance_traces.rs
- [[is_safe_identifier()]] - code - api_rust/src/jobs/cleanup.rs
- [[purge_soft_deleted()]] - code - api_rust/src/jobs/cleanup.rs
- [[safe_identifier_accepts_snake_case()]] - code - api_rust/src/jobs/cleanup.rs
- [[safe_identifier_rejects_injection_attempts()]] - code - api_rust/src/jobs/cleanup.rs
- [[secs_until_utc()]] - code - api_rust/src/jobs/cron.rs
- [[start_cron()]] - code - api_rust/src/jobs/cron.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Community_115
SORT file.name ASC
```

## Connections to other communities

- 3 edges to [[_COMMUNITY_Cycles Rust API]]
- 1 edge to [[_COMMUNITY_Community 129]]
- 1 edge to [[_COMMUNITY_Auth Rust API]]
- 1 edge to [[_COMMUNITY_Community 229]]
- 1 edge to [[_COMMUNITY_Community 181]]

## Top bridge nodes

- [[start_cron()]] - degree 15, connects to 3 communities
- [[delete_issue_description_versions()]] - degree 3, connects to 1 community
- [[delete_page_versions()]] - degree 3, connects to 1 community
- [[find_tables_with_deleted_at()]] - degree 3, connects to 1 community
- [[enqueue_issue_automation()]] - degree 3, connects to 1 community
