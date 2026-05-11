---
type: community
cohesion: 0.16
members: 14
---

# Migrate Filters

**Cohesion:** 0.16 - loosely connected
**Members:** 14 nodes

## Members

- [[0107_migrate_filters_to_rich_filters.py]] - code - api/plane/db/migrations/0107_migrate_filters_to_rich_filters.py
- [[Clear rich_filters field for provided models (for reverse migration).      Args]] - rationale - api/plane/utils/filters/filter_migrations.py
- [[Migrate filters to rich_filters for a single model.      Args         model_cla]] - rationale - api/plane/utils/filters/filter_migrations.py
- [[Migrate legacy filters to rich_filters format for all models that have both fiel]] - rationale - api/plane/db/migrations/0107_migrate_filters_to_rich_filters.py
- [[Migrate legacy filters to rich_filters format for provided models.      Args]] - rationale - api/plane/utils/filters/filter_migrations.py
- [[Migration_56]] - code - api/plane/db/migrations/0107_migrate_filters_to_rich_filters.py
- [[Reverse migration to clear rich_filters field for all models.]] - rationale - api/plane/db/migrations/0107_migrate_filters_to_rich_filters.py
- [[clear_models_rich_filters()]] - code - api/plane/utils/filters/filter_migrations.py
- [[converters.py]] - code - api/plane/utils/filters/converters.py
- [[filter_migrations.py]] - code - api/plane/utils/filters/filter_migrations.py
- [[migrate_filters_to_rich_filters()]] - code - api/plane/db/migrations/0107_migrate_filters_to_rich_filters.py
- [[migrate_models_filters_to_rich_filters()]] - code - api/plane/utils/filters/filter_migrations.py
- [[migrate_single_model_filters()]] - code - api/plane/utils/filters/filter_migrations.py
- [[reverse_migrate_rich_filters_to_filters()]] - code - api/plane/db/migrations/0107_migrate_filters_to_rich_filters.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Migrate_Filters
SORT file.name ASC
```

## Connections to other communities

- 2 edges to [[_COMMUNITY_Field Validate]]
- 1 edge to [[_COMMUNITY_Excluding Soft]]

## Top bridge nodes

- [[converters.py]] - degree 3, connects to 2 communities
- [[migrate_filters_to_rich_filters()]] - degree 4, connects to 1 community
