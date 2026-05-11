---
type: community
cohesion: 0.18
members: 12
---

# Logger Title

**Cohesion:** 0.18 - loosely connected
**Members:** 12 nodes

## Members

- [[.afterLoadDocument()]] - code - live/src/extensions/title-sync.ts
- [[.afterUnloadDocument()]] - code - live/src/extensions/title-sync.ts
- [[.beforeUnloadDocument()]] - code - live/src/extensions/title-sync.ts
- [[.constructor()_189]] - code - live/src/extensions/logger.ts
- [[.handleTitleChange()]] - code - live/src/extensions/title-sync.ts
- [[.onConfigure()]] - code - live/src/extensions/force-close-handler.ts
- [[.onLoadDocument()]] - code - live/src/extensions/title-sync.ts
- [[ForceCloseHandler]] - code - live/src/extensions/force-close-handler.ts
- [[Logger]] - code - live/src/extensions/logger.ts
- [[TitleSyncExtension]] - code - live/src/extensions/title-sync.ts
- [[index.ts_451]] - code - live/src/extensions/index.ts
- [[logger.ts_1]] - code - live/src/extensions/logger.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Logger_Title
SORT file.name ASC
```

## Connections to other communities

- 5 edges to [[_COMMUNITY_Broadcast Context]]
- 4 edges to [[_COMMUNITY_Close Force]]
- 2 edges to [[_COMMUNITY_Controller Document]]
- 1 edge to [[_COMMUNITY_Workspace Webhook]]

## Top bridge nodes

- [[index.ts_451]] - degree 12, connects to 4 communities
- [[TitleSyncExtension]] - degree 7, connects to 1 community
- [[ForceCloseHandler]] - degree 3, connects to 1 community
- [[.handleTitleChange()]] - degree 2, connects to 1 community
- [[.onLoadDocument()]] - degree 2, connects to 1 community
