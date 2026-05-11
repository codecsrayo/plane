---
type: community
cohesion: 0.17
members: 12
---

# Factory Creating

**Cohesion:** 0.17 - loosely connected
**Members:** 12 nodes

## Members

- [[Factory for creating Project instances]] - rationale - api/plane/tests/factories.py
- [[Factory for creating ProjectMember instances]] - rationale - api/plane/tests/factories.py
- [[Factory for creating User instances]] - rationale - api/plane/tests/factories.py
- [[Factory for creating Workspace instances]] - rationale - api/plane/tests/factories.py
- [[Factory for creating WorkspaceMember instances]] - rationale - api/plane/tests/factories.py
- [[Meta_24]] - code - api/plane/tests/factories.py
- [[ProjectFactory]] - code - api/plane/tests/factories.py
- [[ProjectMemberFactory]] - code - api/plane/tests/factories.py
- [[UserFactory]] - code - api/plane/tests/factories.py
- [[WorkspaceFactory]] - code - api/plane/tests/factories.py
- [[WorkspaceMemberFactory]] - code - api/plane/tests/factories.py
- [[factories.py]] - code - api/plane/tests/factories.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Factory_Creating
SORT file.name ASC
```
