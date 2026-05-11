---
type: community
cohesion: 0.10
members: 26
---

# Task Object

**Cohesion:** 0.10 - loosely connected
**Members:** 26 nodes

## Members

- [[.copy_object()]] - code - api/plane/settings/storage.py
- [[.delete_files()]] - code - api/plane/settings/storage.py
- [[.get_object_metadata()]] - code - api/plane/settings/storage.py
- [[.get_queryset()]] - code - api/plane/space/views/base.py
- [[.upload_file()]] - code - api/plane/settings/storage.py
- [[Copy an S3 object to a new location]] - rationale - api/plane/settings/storage.py
- [[Get the metadata for an S3 object]] - rationale - api/plane/settings/storage.py
- [[Upload a file directly to S3]] - rationale - api/plane/settings/storage.py
- [[archive_and_close_old_issues()]] - code - api/plane/bgtasks/issue_automation_task.py
- [[archive_old_issues()]] - code - api/plane/bgtasks/issue_automation_task.py
- [[close_old_issues()]] - code - api/plane/bgtasks/issue_automation_task.py
- [[decrypt_data()]] - code - api/plane/license/utils/encryption.py
- [[derive_key()]] - code - api/plane/license/utils/encryption.py
- [[encrypt_data()]] - code - api/plane/license/utils/encryption.py
- [[encryption.py]] - code - api/plane/license/utils/encryption.py
- [[exception_logger.py]] - code - api/plane/utils/exception_logger.py
- [[get_asset_object_metadata()]] - code - api/plane/bgtasks/storage_metadata_task.py
- [[issue_automation_task.py]] - code - api/plane/bgtasks/issue_automation_task.py
- [[log_exception()]] - code - api/plane/utils/exception_logger.py
- [[log_issue_description_version()]] - code - api/plane/db/models/issue.py
- [[log_issue_version()]] - code - api/plane/db/models/issue.py
- [[page_version_task.py]] - code - api/plane/bgtasks/page_version_task.py
- [[recent_visited_task()]] - code - api/plane/bgtasks/recent_visited_task.py
- [[recent_visited_task.py]] - code - api/plane/bgtasks/recent_visited_task.py
- [[storage_metadata_task.py]] - code - api/plane/bgtasks/storage_metadata_task.py
- [[track_page_version()]] - code - api/plane/bgtasks/page_version_task.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Task_Object
SORT file.name ASC
```

## Connections to other communities

- 10 edges to [[_COMMUNITY_Email Task]]
- 7 edges to [[_COMMUNITY_Endpoint Issue]]
- 4 edges to [[_COMMUNITY_Copy Test]]
- 3 edges to [[_COMMUNITY_Mongo Process]]
- 3 edges to [[_COMMUNITY_Webhook Model]]
- 3 edges to [[_COMMUNITY_Issue Version]]
- 2 edges to [[_COMMUNITY_Endpoint User]]
- 2 edges to [[_COMMUNITY_Handle Exception]]
- 2 edges to [[_COMMUNITY_Provider Endpoint]]
- 2 edges to [[_COMMUNITY_Work Link]]
- 2 edges to [[_COMMUNITY_Generate Analytic]]
- 2 edges to [[_COMMUNITY_Transform Logs]]
- 2 edges to [[_COMMUNITY_Issue Sync]]
- 2 edges to [[_COMMUNITY_Check User]]
- 2 edges to [[_COMMUNITY_Instance Endpoint]]
- 1 edge to [[_COMMUNITY_Email Process]]
- 1 edge to [[_COMMUNITY_Event Tracking]]
- 1 edge to [[_COMMUNITY_Transaction Task]]
- 1 edge to [[_COMMUNITY_Valid Test]]
- 1 edge to [[_COMMUNITY_Provider Upload]]
- 1 edge to [[_COMMUNITY_Issue Description]]
- 1 edge to [[_COMMUNITY_Issue Description]]
- 1 edge to [[_COMMUNITY_Middleware Logger]]
- 1 edge to [[_COMMUNITY_Serializer Validate]]
- 1 edge to [[_COMMUNITY_Test Validate]]
- 1 edge to [[_COMMUNITY_User Oauth]]
- 1 edge to [[_COMMUNITY_Command Issue]]
- 1 edge to [[_COMMUNITY_Endpoint Magic]]

## Top bridge nodes

- [[log_exception()]] - degree 64, connects to 24 communities
- [[decrypt_data()]] - degree 5, connects to 2 communities
- [[encrypt_data()]] - degree 5, connects to 2 communities
- [[get_asset_object_metadata()]] - degree 3, connects to 1 community
- [[.copy_object()]] - degree 3, connects to 1 community
