---
type: community
cohesion: 0.11
members: 27
---

# Upload Detect

**Cohesion:** 0.11 - loosely connected
**Members:** 27 nodes

## Members

- [[.cancelUpload()_1]] - code - services/src/file/file-upload.service.ts
- [[.cancelUpload()]] - code - services/src/file/sites-file.service.ts
- [[.constructor()_18]] - code - services/src/file/file.service.ts
- [[.constructor()_17]] - code - services/src/file/file-upload.service.ts
- [[.constructor()_16]] - code - services/src/file/sites-file.service.ts
- [[.deleteNewAsset()]] - code - services/src/file/file.service.ts
- [[.deleteOldEditorAsset()]] - code - services/src/file/file.service.ts
- [[.duplicateAssets()]] - code - services/src/file/file.service.ts
- [[.restoreNewAsset()]] - code - services/src/file/sites-file.service.ts
- [[.restoreOldEditorAsset()]] - code - services/src/file/file.service.ts
- [[.updateAssetUploadStatus()]] - code - services/src/file/sites-file.service.ts
- [[.updateBulkAssetsUploadStatus()]] - code - services/src/file/sites-file.service.ts
- [[.uploadAsset()]] - code - services/src/file/sites-file.service.ts
- [[.uploadFile()]] - code - services/src/file/file-upload.service.ts
- [[FileService]] - code - services/src/file/file.service.ts
- [[FileUploadService]] - code - services/src/file/file-upload.service.ts
- [[SitesFileService]] - code - services/src/file/sites-file.service.ts
- [[detectMimeTypeFromSignature()]] - code - services/src/file/helper.ts
- [[file-upload.service.ts]] - code - services/src/file/file-upload.service.ts
- [[file.service.ts]] - code - services/src/file/file.service.ts
- [[generateFileUploadPayload()]] - code - services/src/file/helper.ts
- [[getAssetIdFromUrl()]] - code - services/src/file/helper.ts
- [[getFileMetaDataForUpload()]] - code - services/src/file/helper.ts
- [[helper.ts]] - code - services/src/file/helper.ts
- [[sites-file.service.ts]] - code - services/src/file/sites-file.service.ts
- [[validateAndDetectFileType()]] - code - services/src/file/helper.ts
- [[validateFilename()]] - code - services/src/file/helper.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Upload_Detect
SORT file.name ASC
```

## Connections to other communities

- 2 edges to [[_COMMUNITY_Sites Cycle]]

## Top bridge nodes

- [[file.service.ts]] - degree 5, connects to 1 community
- [[file-upload.service.ts]] - degree 3, connects to 1 community
