---
type: community
cohesion: 0.14
members: 14
---

# Payload Integration

**Cohesion:** 0.14 - loosely connected
**Members:** 14 nodes

## Members

- [[IAppIntegration]] - code - types/src/integration.ts
- [[IGithubRepo]] - code - types/src/integration.ts
- [[IGithubRepoSync]] - code - types/src/integration.ts
- [[IGithubRepositoriesResponse]] - code - types/src/integration.ts
- [[ISlackIntegration]] - code - types/src/integration.ts
- [[ISlackIntegrationData]] - code - types/src/integration.ts
- [[IWorkspaceIntegration]] - code - types/src/integration.ts
- [[TGithubInstallPayload]] - code - types/src/integration.ts
- [[TGithubRepoSyncCreatePayload]] - code - types/src/integration.ts
- [[TGitlabInstallPayload]] - code - types/src/integration.ts
- [[TProviderInstallPayload]] - code - types/src/integration.ts
- [[TSlackChannelCreatePayload]] - code - types/src/integration.ts
- [[TSlackInstallPayload]] - code - types/src/integration.ts
- [[integration.ts]] - code - types/src/integration.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Payload_Integration
SORT file.name ASC
```
