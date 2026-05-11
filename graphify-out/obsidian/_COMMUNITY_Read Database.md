---
type: community
cohesion: 0.18
members: 13
---

# Read Database

**Cohesion:** 0.18 - loosely connected
**Members:** 13 nodes

## Members

- [[.allow_migrate()]] - code - api/plane/utils/core/dbrouters.py
- [[.db_for_read()]] - code - api/plane/utils/core/dbrouters.py
- [[.db_for_write()]] - code - api/plane/utils/core/dbrouters.py
- [[Check if the current request should use read replica database.     This function]] - rationale - api/plane/utils/core/request_scope.py
- [[Database router that directs read operations to replica when appropriate.     Th]] - rationale - api/plane/utils/core/dbrouters.py
- [[Determine which database to use for read operations.         Args             m]] - rationale - api/plane/utils/core/dbrouters.py
- [[Determine which database to use for write operations.         All write operatio]] - rationale - api/plane/utils/core/dbrouters.py
- [[Ensure migrations only run on the primary database.         Args             db]] - rationale - api/plane/utils/core/dbrouters.py
- [[ReadReplicaRouter]] - code - api/plane/utils/core/dbrouters.py
- [[__init__.py_43]] - code - api/plane/utils/core/**init**.py
- [[dbrouters.py]] - code - api/plane/utils/core/dbrouters.py
- [[request_scope.py]] - code - api/plane/utils/core/request_scope.py
- [[should_use_read_replica()]] - code - api/plane/utils/core/request_scope.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Read_Database
SORT file.name ASC
```

## Connections to other communities

- 2 edges to [[_COMMUNITY_Read Replica]]

## Top bridge nodes

- [[request_scope.py]] - degree 5, connects to 1 community
