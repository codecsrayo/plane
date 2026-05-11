---
type: community
cohesion: 0.25
members: 8
---

# Test Should

**Cohesion:** 0.25 - loosely connected
**Members:** 8 nodes

## Members

- [[.test_should_use_read_replica_with_false_attribute()]] - code - api/plane/tests/unit/middleware/test_db_routing.py
- [[.test_should_use_read_replica_with_no_attribute_defaults_false()]] - code - api/plane/tests/unit/middleware/test_db_routing.py
- [[.test_should_use_read_replica_with_true_attribute()]] - code - api/plane/tests/unit/middleware/test_db_routing.py
- [[Test _should_use_read_replica defaults to False for missing attr.]] - rationale - api/plane/tests/unit/middleware/test_db_routing.py
- [[Test _should_use_read_replica returns False for False attribute.]] - rationale - api/plane/tests/unit/middleware/test_db_routing.py
- [[Test _should_use_read_replica returns True for True attribute.]] - rationale - api/plane/tests/unit/middleware/test_db_routing.py
- [[Test cases for replica decision logic methods.]] - rationale - api/plane/tests/unit/middleware/test_db_routing.py
- [[TestReplicaDecisionLogic]] - code - api/plane/tests/unit/middleware/test_db_routing.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Test_Should
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Test Fixture]]
- 1 edge to [[_COMMUNITY_Read Routing]]

## Top bridge nodes

- [[TestReplicaDecisionLogic]] - degree 6, connects to 2 communities
