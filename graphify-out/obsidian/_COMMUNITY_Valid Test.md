---
type: community
cohesion: 0.18
members: 12
---

# Valid Test

**Cohesion:** 0.18 - loosely connected
**Members:** 12 nodes

## Members

- [[.test_is_valid_uuid_with_invalid_uuid()]] - code - api/plane/tests/unit/utils/test_uuid.py
- [[.test_is_valid_uuid_with_valid_uuid()]] - code - api/plane/tests/unit/utils/test_uuid.py
- [[Check if a string is a valid UUID version 4]] - rationale - api/plane/utils/uuid.py
- [[Test is_valid_uuid with a valid UUID]] - rationale - api/plane/tests/unit/utils/test_uuid.py
- [[Test is_valid_uuid with invalid UUID strings]] - rationale - api/plane/tests/unit/utils/test_uuid.py
- [[Test the UUID utilities]] - rationale - api/plane/tests/unit/utils/test_uuid.py
- [[TestUUIDUtils]] - code - api/plane/tests/unit/utils/test_uuid.py
- [[is_valid_uuid()]] - code - api/plane/utils/uuid.py
- [[issue_activity()]] - code - api/plane/bgtasks/issue_activities_task.py
- [[test_uuid.py]] - code - api/plane/tests/unit/utils/test_uuid.py
- [[track_parent()]] - code - api/plane/bgtasks/issue_activities_task.py
- [[track_state()]] - code - api/plane/bgtasks/issue_activities_task.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Valid_Test
SORT file.name ASC
```

## Connections to other communities

- 5 edges to [[_COMMUNITY_Issue Track]]
- 3 edges to [[_COMMUNITY_Uuid Test]]
- 1 edge to [[_COMMUNITY_Issue Apply]]
- 1 edge to [[_COMMUNITY_Test Magic]]
- 1 edge to [[_COMMUNITY_Task Object]]

## Top bridge nodes

- [[issue_activity()]] - degree 5, connects to 4 communities
- [[is_valid_uuid()]] - degree 9, connects to 2 communities
- [[TestUUIDUtils]] - degree 6, connects to 1 community
- [[track_parent()]] - degree 2, connects to 1 community
- [[track_state()]] - degree 2, connects to 1 community
