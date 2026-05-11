---
type: community
cohesion: 0.22
members: 10
---

# State Multi

**Cohesion:** 0.22 - loosely connected
**Members:** 10 nodes

## Members

- [[TCreateStateFilterParams]] - code - utils/src/work-item-filters/configs/filters/state.ts
- [[TCreateStateGroupFilterParams]] - code - utils/src/work-item-filters/configs/filters/state.ts
- [[getMemberMultiSelectConfig()]] - code - utils/src/rich-filters/factories/configs/properties/shared.ts
- [[getMultiSelectConfig()]] - code - utils/src/rich-filters/factories/configs/core.ts
- [[getProjectMultiSelectConfig()_1]] - code - utils/src/rich-filters/factories/configs/properties/shared.ts
- [[getStateFilterConfig()]] - code - utils/src/work-item-filters/configs/filters/state.ts
- [[getStateGroupFilterConfig()]] - code - utils/src/work-item-filters/configs/filters/state.ts
- [[getStateGroupMultiSelectConfig()]] - code - utils/src/work-item-filters/configs/filters/state.ts
- [[getStateMultiSelectConfig()]] - code - utils/src/work-item-filters/configs/filters/state.ts
- [[state.ts_4]] - code - utils/src/work-item-filters/configs/filters/state.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/State_Multi
SORT file.name ASC
```

## Connections to other communities

- 4 edges to [[_COMMUNITY_Params Date]]
- 2 edges to [[_COMMUNITY_Cycle Date]]
- 2 edges to [[_COMMUNITY_Config Date]]
- 1 edge to [[_COMMUNITY_Label Multi]]
- 1 edge to [[_COMMUNITY_Priority Multi]]
- 1 edge to [[_COMMUNITY_Multi Select]]
- 1 edge to [[_COMMUNITY_Project Multi]]

## Top bridge nodes

- [[getMultiSelectConfig()]] - degree 12, connects to 7 communities
- [[state.ts_4]] - degree 7, connects to 1 community
- [[getMemberMultiSelectConfig()]] - degree 3, connects to 1 community
- [[getProjectMultiSelectConfig()_1]] - degree 2, connects to 1 community
