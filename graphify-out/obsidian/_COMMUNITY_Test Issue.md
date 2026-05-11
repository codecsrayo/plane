---
type: community
cohesion: 0.29
members: 8
---

# Test Issue

**Cohesion:** 0.29 - loosely connected
**Members:** 8 nodes

## Members

- [[.get_assignees()]] - code - api/plane/app/serializers/workspace.py
- [[.get_project_identifier()]] - code - api/plane/app/serializers/workspace.py
- [[.test_issue_recent_visit_serializer_fields()]] - code - api/plane/tests/unit/serializers/test_issue_recent_visit.py
- [[IssueRecentVisitSerializer]] - code - api/plane/app/serializers/workspace.py
- [[Test that the serializer includes the correct fields]] - rationale - api/plane/tests/unit/serializers/test_issue_recent_visit.py
- [[Test the IssueRecentVisitSerializer]] - rationale - api/plane/tests/unit/serializers/test_issue_recent_visit.py
- [[TestIssueRecentVisitSerializer]] - code - api/plane/tests/unit/serializers/test_issue_recent_visit.py
- [[test_issue_recent_visit.py]] - code - api/plane/tests/unit/serializers/test_issue_recent_visit.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Test_Issue
SORT file.name ASC
```

## Connections to other communities

- 3 edges to [[_COMMUNITY_Serializer Project]]
- 1 edge to [[_COMMUNITY_Serializer Issue]]
- 1 edge to [[_COMMUNITY_Serializer Issue]]

## Top bridge nodes

- [[IssueRecentVisitSerializer]] - degree 9, connects to 3 communities
