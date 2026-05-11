---
type: community
cohesion: 0.24
members: 11
---

# Community 452

**Cohesion:** 0.24 - loosely connected
**Members:** 11 nodes

## Members
- [[Fallback to logging to PostgreSQL if MongoDB is unavailable.]] - rationale - api/plane/bgtasks/logger_task.py
- [[Logs the request to MongoDB if available.]] - rationale - api/plane/bgtasks/logger_task.py
- [[Process logs to save to MongoDB or Postgres based on the configuration]] - rationale - api/plane/bgtasks/logger_task.py
- [[Returns the MongoDB collection for external API activity logs.]] - rationale - api/plane/bgtasks/logger_task.py
- [[Safely decodes requestresponse body content, handling binary data.     Returns]] - rationale - api/plane/bgtasks/logger_task.py
- [[get_mongo_collection()]] - code - api/plane/bgtasks/logger_task.py
- [[log_to_mongo()]] - code - api/plane/bgtasks/logger_task.py
- [[log_to_postgres()]] - code - api/plane/bgtasks/logger_task.py
- [[logger_task.py]] - code - api/plane/bgtasks/logger_task.py
- [[process_logs()]] - code - api/plane/bgtasks/logger_task.py
- [[safe_decode_body()]] - code - api/plane/bgtasks/logger_task.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Community_452
SORT file.name ASC
```

## Connections to other communities
- 3 edges to [[_COMMUNITY_Community 48]]

## Top bridge nodes
- [[log_to_mongo()]] - degree 5, connects to 1 community
- [[get_mongo_collection()]] - degree 4, connects to 1 community
- [[log_to_postgres()]] - degree 4, connects to 1 community