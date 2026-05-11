---
type: community
cohesion: 0.25
members: 8
---

# Issues Modules

**Cohesion:** 0.25 - loosely connected
**Members:** 8 nodes

## Members

- [[.addIssuesToModule()]] - code - services/src/module/operations.service.ts
- [[.addModuleToFavorites()]] - code - services/src/module/operations.service.ts
- [[.addModulesToIssue()]] - code - services/src/module/operations.service.ts
- [[.removeIssuesFromModuleBulk()]] - code - services/src/module/operations.service.ts
- [[.removeModuleFromFavorites()]] - code - services/src/module/operations.service.ts
- [[.removeModulesFromIssueBulk()]] - code - services/src/module/operations.service.ts
- [[ModuleOperationService]] - code - services/src/module/operations.service.ts
- [[operations.service.ts]] - code - services/src/module/operations.service.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Issues_Modules
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Sites Cycle]]

## Top bridge nodes

- [[operations.service.ts]] - degree 2, connects to 1 community
