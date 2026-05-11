---
type: community
cohesion: 0.17
members: 17
---

# Config Only

**Cohesion:** 0.17 - loosely connected
**Members:** 17 nodes

## Members

- [[.admin_base()]] - code - api_rust/src/config.rs
- [[.app_base()]] - code - api_rust/src/config.rs
- [[.app_base_url_only()]] - code - api_rust/src/config.rs
- [[.from_env()]] - code - api_rust/src/config.rs
- [[.space_base()]] - code - api_rust/src/config.rs
- [[Config]] - code - api_rust/src/config.rs
- [[app_base_includes_path()]] - code - api_rust/src/config.rs
- [[app_base_url_only_defaults_to_root_slash()]] - code - api_rust/src/config.rs
- [[app_base_url_only_falls_back_to_web_url()]] - code - api_rust/src/config.rs
- [[app_base_url_only_ignores_app_base_path()]] - code - api_rust/src/config.rs
- [[build_database_url()]] - code - api_rust/src/config.rs
- [[build_redis_url()]] - code - api_rust/src/config.rs
- [[config.rs]] - code - api_rust/src/config.rs
- [[config_with()]] - code - api_rust/src/config.rs
- [[delete_old_s3_link()]] - code - api/plane/bgtasks/exporter_expired_task.py
- [[exporter_expired_task.py]] - code - api/plane/bgtasks/exporter_expired_task.py
- [[required()]] - code - api_rust/src/config.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Config_Only
SORT file.name ASC
```

## Connections to other communities

- 2 edges to [[_COMMUNITY_Password Email]]
- 1 edge to [[_COMMUNITY_Provider Upload]]

## Top bridge nodes

- [[Config]] - degree 8, connects to 1 community
- [[config_with()]] - degree 6, connects to 1 community
- [[.from_env()]] - degree 5, connects to 1 community
