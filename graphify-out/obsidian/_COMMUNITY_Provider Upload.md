---
type: community
cohesion: 0.32
members: 8
---

# Provider Upload

**Cohesion:** 0.32 - loosely connected
**Members:** 8 nodes

## Members

- [[TODO Change the upload_to_s3 function to use the new storage method with entr]] - rationale - api/plane/bgtasks/export_task.py
- [[Create a ZIP file from the provided files.]] - rationale - api/plane/bgtasks/export_task.py
- [[Export issues from the workspace.     provider (str) The provider to export the]] - rationale - api/plane/bgtasks/export_task.py
- [[Upload a ZIP file to S3 and generate a presigned URL.]] - rationale - api/plane/bgtasks/export_task.py
- [[create_zip_file()]] - code - api/plane/bgtasks/export_task.py
- [[export_task.py]] - code - api/plane/bgtasks/export_task.py
- [[issue_export_task()]] - code - api/plane/bgtasks/export_task.py
- [[upload_to_s3()]] - code - api/plane/bgtasks/export_task.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Provider_Upload
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Config Only]]
- 1 edge to [[_COMMUNITY_Exporter Formatter]]
- 1 edge to [[_COMMUNITY_Task Object]]

## Top bridge nodes

- [[issue_export_task()]] - degree 6, connects to 2 communities
- [[upload_to_s3()]] - degree 4, connects to 1 community
