---
type: community
cohesion: 0.07
members: 35
---

# Transform Logs

**Cohesion:** 0.07 - loosely connected
**Members:** 35 nodes

## Members

- [[Delete excess issue description versions.]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[Delete excess page versions.]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[Delete old API activity logs.]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[Delete old email notification logs.]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[Delete old webhook logs]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[Generic function to process cleanup tasks.      Args         queryset_func Fun]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[Get API logs older than cutoff days.]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[Get MongoDB collection if available, otherwise return None.]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[Get email logs older than cutoff days.]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[Get email logs older than cutoff days._1]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[Get issue description versions beyond the maximum allowed (20 per issue).]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[Inserts a batch of records into MongoDB and deletes the corresponding rows from]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[Transfer webhook logs to a new destination.]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[Transform API activity log record.]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[Transform email notification log record.]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[Transform issue description version record.]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[Transform page version record.]] - rationale - api/plane/bgtasks/cleanup_task.py
- [[cleanup_task.py]] - code - api/plane/bgtasks/cleanup_task.py
- [[delete_api_logs()]] - code - api/plane/bgtasks/cleanup_task.py
- [[delete_email_notification_logs()]] - code - api/plane/bgtasks/cleanup_task.py
- [[delete_issue_description_versions()]] - code - api/plane/bgtasks/cleanup_task.py
- [[delete_page_versions()]] - code - api/plane/bgtasks/cleanup_task.py
- [[delete_webhook_logs()]] - code - api/plane/bgtasks/cleanup_task.py
- [[flush_to_mongo_and_delete()]] - code - api/plane/bgtasks/cleanup_task.py
- [[get_api_logs_queryset()]] - code - api/plane/bgtasks/cleanup_task.py
- [[get_email_logs_queryset()]] - code - api/plane/bgtasks/cleanup_task.py
- [[get_issue_description_versions_queryset()]] - code - api/plane/bgtasks/cleanup_task.py
- [[get_mongo_collection()_1]] - code - api/plane/bgtasks/cleanup_task.py
- [[get_webhook_logs_queryset()]] - code - api/plane/bgtasks/cleanup_task.py
- [[process_cleanup_task()]] - code - api/plane/bgtasks/cleanup_task.py
- [[transform_api_log()]] - code - api/plane/bgtasks/cleanup_task.py
- [[transform_email_log()]] - code - api/plane/bgtasks/cleanup_task.py
- [[transform_issue_description_version()]] - code - api/plane/bgtasks/cleanup_task.py
- [[transform_page_version()]] - code - api/plane/bgtasks/cleanup_task.py
- [[transform_webhook_log()]] - code - api/plane/bgtasks/cleanup_task.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Transform_Logs
SORT file.name ASC
```

## Connections to other communities

- 2 edges to [[_COMMUNITY_Endpoint User]]
- 2 edges to [[_COMMUNITY_Task Object]]
- 1 edge to [[_COMMUNITY_Test Validate]]

## Top bridge nodes

- [[flush_to_mongo_and_delete()]] - degree 5, connects to 2 communities
- [[cleanup_task.py]] - degree 18, connects to 1 community
- [[get_mongo_collection()_1]] - degree 4, connects to 1 community
- [[get_issue_description_versions_queryset()]] - degree 3, connects to 1 community
