---
type: community
cohesion: 0.16
members: 16
---

# Middleware Logger

**Cohesion:** 0.16 - loosely connected
**Members:** 16 nodes

## Members

- [[.__call__()_2]] - code - api/plane/middleware/logger.py
- [[.__call__()_1]] - code - api/plane/middleware/logger.py
- [[.__init__()_7]] - code - api/plane/middleware/logger.py
- [[.__init__()_6]] - code - api/plane/middleware/logger.py
- [[._safe_decode_body()]] - code - api/plane/middleware/logger.py
- [[._should_log_route()]] - code - api/plane/middleware/logger.py
- [[.process_request()]] - code - api/plane/middleware/logger.py
- [[.save_user_data()]] - code - api/plane/authentication/adapter/base.py
- [[APITokenLogMiddleware]] - code - api/plane/middleware/logger.py
- [[Determines whether a route should be logged based on the request and status code]] - rationale - api/plane/middleware/logger.py
- [[Middleware to log External API requests to MongoDB or PostgreSQL.]] - rationale - api/plane/middleware/logger.py
- [[RequestLoggerMiddleware]] - code - api/plane/middleware/logger.py
- [[Safely decodes requestresponse body content, handling binary data.         Retu]] - rationale - api/plane/middleware/logger.py
- [[get_client_ip()]] - code - api/plane/utils/ip_address.py
- [[ip_address.py]] - code - api/plane/utils/ip_address.py
- [[logger.py]] - code - api/plane/middleware/logger.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Middleware_Logger
SORT file.name ASC
```

## Connections to other communities

- 2 edges to [[_COMMUNITY_Check User]]
- 1 edge to [[_COMMUNITY_Task Object]]
- 1 edge to [[_COMMUNITY_Host Utility]]
- 1 edge to [[_COMMUNITY_Path Redirection]]
- 1 edge to [[_COMMUNITY_Endpoint Sign]]

## Top bridge nodes

- [[get_client_ip()]] - degree 7, connects to 3 communities
- [[.process_request()]] - degree 5, connects to 1 community
- [[.save_user_data()]] - degree 3, connects to 1 community
