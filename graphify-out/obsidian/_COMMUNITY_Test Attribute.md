---
type: community
cohesion: 0.25
members: 8
---

# Test Attribute

**Cohesion:** 0.25 - loosely connected
**Members:** 8 nodes

## Members

- [[.test_get_use_replica_attribute_with_attribute_error()]] - code - api/plane/tests/unit/middleware/test_db_routing.py
- [[.test_multiple_exception_calls_are_safe()]] - code - api/plane/tests/unit/middleware/test_db_routing.py
- [[.test_process_view_with_none_view_func()]] - code - api/plane/tests/unit/middleware/test_db_routing.py
- [[Test _get_use_replica_attribute with view that raises AttributeError.]] - rationale - api/plane/tests/unit/middleware/test_db_routing.py
- [[Test edge cases and error conditions.]] - rationale - api/plane/tests/unit/middleware/test_db_routing.py
- [[Test process_view handles None view_func gracefully.]] - rationale - api/plane/tests/unit/middleware/test_db_routing.py
- [[Test that multiple calls to process_exception don't cause issues.]] - rationale - api/plane/tests/unit/middleware/test_db_routing.py
- [[TestEdgeCases]] - code - api/plane/tests/unit/middleware/test_db_routing.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Test_Attribute
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Test Fixture]]
- 1 edge to [[_COMMUNITY_Read Routing]]

## Top bridge nodes

- [[TestEdgeCases]] - degree 6, connects to 2 communities
