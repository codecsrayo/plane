---
type: community
cohesion: 1.00
members: 1
---

# Attachments Issue

**Cohesion:** 1.00 - tightly connected
**Members:** 1 nodes

## Members

- [[List issue attachments          List all attachments for an issue.]] - rationale - api/plane/api/views/issue.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Attachments_Issue
SORT file.name ASC
```
