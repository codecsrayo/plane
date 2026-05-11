---
type: community
cohesion: 0.10
members: 21
---

# Workspace Upload

**Cohesion:** 0.10 - loosely connected
**Members:** 21 nodes

## Members

- [[.cancelUpload()_1]] - code - services/file.service.ts
- [[.checkIfAssetExists()]] - code - services/file.service.ts
- [[.constructor()_90]] - code - services/file.service.ts
- [[.deleteNewAsset()]] - code - services/file.service.ts
- [[.deleteOldUserAsset()]] - code - services/file.service.ts
- [[.deleteOldWorkspaceAsset()]] - code - services/file.service.ts
- [[.deleteUserAsset()]] - code - services/file.service.ts
- [[.deleteWorkspaceAsset()]] - code - services/file.service.ts
- [[.duplicateAsset()]] - code - services/file.service.ts
- [[.getUnsplashImages()]] - code - services/file.service.ts
- [[.restoreNewAsset()]] - code - services/file.service.ts
- [[.restoreOldEditorAsset()]] - code - services/file.service.ts
- [[.updateBulkProjectAssetsUploadStatus()]] - code - services/file.service.ts
- [[.updateBulkWorkspaceAssetsUploadStatus()]] - code - services/file.service.ts
- [[.updateProjectAssetUploadStatus()]] - code - services/file.service.ts
- [[.updateUserAssetUploadStatus()]] - code - services/file.service.ts
- [[.updateWorkspaceAssetUploadStatus()]] - code - services/file.service.ts
- [[.uploadProjectAsset()]] - code - services/file.service.ts
- [[.uploadUserAsset()]] - code - services/file.service.ts
- [[.uploadWorkspaceAsset()]] - code - services/file.service.ts
- [[FileService]] - code - services/file.service.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Workspace_Upload
SORT file.name ASC
```

## Connections to other communities

- 2 edges to [[_COMMUNITY_Editor Asset]]
- 2 edges to [[_COMMUNITY_Image Upload]]
- 1 edge to [[_COMMUNITY_API Services]]
- 1 edge to [[_COMMUNITY_Project Feature]]
- 1 edge to [[_COMMUNITY_Issue Identifiers]]
- 1 edge to [[_COMMUNITY_Issue Inbox]]
- 1 edge to [[_COMMUNITY_Comment Card]]
- 1 edge to [[_COMMUNITY_Workspace Timezone]]

## Top bridge nodes

- [[FileService]] - degree 30, connects to 8 communities
