---
type: community
cohesion: 0.15
members: 15
---

# Read Routing

**Cohesion:** 0.15 - loosely connected
**Members:** 15 nodes

## Members

- [[.__init__()_8]] - code - api/plane/middleware/db_routing.py
- [[._get_use_replica_attribute()]] - code - api/plane/middleware/db_routing.py
- [[._should_use_read_replica()]] - code - api/plane/middleware/db_routing.py
- [[.process_view()]] - code - api/plane/middleware/db_routing.py
- [[Determine if the view should use read replica based on its configuration.]] - rationale - api/plane/middleware/db_routing.py
- [[Extract the use_read_replica attribute from various view types.         Args]] - rationale - api/plane/middleware/db_routing.py
- [[Hook called just before Django calls the view.         This is more efficient th]] - rationale - api/plane/middleware/db_routing.py
- [[Initialize the middleware with the next middlewareview in the chain.         Ar]] - rationale - api/plane/middleware/db_routing.py
- [[Middleware for intelligent database routing to read replicas.     Routing Logic]] - rationale - api/plane/middleware/db_routing.py
- [[ReadReplicaRoutingMiddleware]] - code - api/plane/middleware/db_routing.py
- [[Test cases for exception handling and cleanup.]] - rationale - api/plane/tests/unit/middleware/test_db_routing.py
- [[Test middleware with real DjangoDRF view classes.]] - rationale - api/plane/tests/unit/middleware/test_db_routing.py
- [[TestExceptionHandling]] - code - api/plane/tests/unit/middleware/test_db_routing.py
- [[TestRealViewIntegration]] - code - api/plane/tests/unit/middleware/test_db_routing.py
- [[db_routing.py]] - code - api/plane/middleware/db_routing.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Read_Routing
SORT file.name ASC
```

## Connections to other communities

- 3 edges to [[_COMMUNITY_Test Fixture]]
- 3 edges to [[_COMMUNITY_Read Replica]]
- 1 edge to [[_COMMUNITY_Test Middleware]]
- 1 edge to [[_COMMUNITY_Test Method]]
- 1 edge to [[_COMMUNITY_Test Should]]
- 1 edge to [[_COMMUNITY_Test Attribute]]
- 1 edge to [[_COMMUNITY_Test Attribute]]

## Top bridge nodes

- [[ReadReplicaRoutingMiddleware]] - degree 16, connects to 7 communities
- [[.process_view()]] - degree 4, connects to 1 community
- [[TestExceptionHandling]] - degree 3, connects to 1 community
- [[TestRealViewIntegration]] - degree 3, connects to 1 community
