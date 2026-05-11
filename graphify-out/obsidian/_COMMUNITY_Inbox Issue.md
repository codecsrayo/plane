---
type: community
cohesion: 0.50
members: 4
---

# Inbox Issue

**Cohesion:** 0.50 - moderately connected
**Members:** 4 nodes

## Members

- [[CreateProjectModal()]] - code - components/project/create-project-modal.tsx
- [[CreateWebhookModal()]] - code - components/web-hooks/create-webhook-modal.tsx
- [[InboxIssueCreateModalRoot()]] - code - components/inbox/modals/create-modal/modal.tsx
- [[useKeypress()]] - code - hooks/use-keypress.tsx

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Inbox_Issue
SORT file.name ASC
```

## Connections to other communities

- 2 edges to [[_COMMUNITY_Issue Inbox]]
- 1 edge to [[_COMMUNITY_Module Forms]]
- 1 edge to [[_COMMUNITY_Project Projects]]
- 1 edge to [[_COMMUNITY_Project Feature]]
- 1 edge to [[_COMMUNITY_Webhook Generated]]

## Top bridge nodes

- [[useKeypress()]] - degree 5, connects to 2 communities
- [[CreateProjectModal()]] - degree 3, connects to 2 communities
- [[InboxIssueCreateModalRoot()]] - degree 2, connects to 1 community
- [[CreateWebhookModal()]] - degree 2, connects to 1 community
