---
type: community
cohesion: 0.25
members: 9
---

# Mongo Connection

**Cohesion:** 0.25 - loosely connected
**Members:** 9 nodes

## Members

- [[.__new__()]] - code - api/plane/settings/mongo.py
- [[A singleton class that manages MongoDB connections.      This class ensures only]] - rationale - api/plane/settings/mongo.py
- [[Creates a new instance of MongoConnection if one doesn't exist.          Returns]] - rationale - api/plane/settings/mongo.py
- [[MongoConnection]] - code - api/plane/settings/mongo.py
- [[get_client()]] - code - api/plane/settings/mongo.py
- [[get_collection()]] - code - api/plane/settings/mongo.py
- [[get_db()]] - code - api/plane/settings/mongo.py
- [[is_configured()]] - code - api/plane/settings/mongo.py
- [[mongo.py]] - code - api/plane/settings/mongo.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Mongo_Connection
SORT file.name ASC
```
