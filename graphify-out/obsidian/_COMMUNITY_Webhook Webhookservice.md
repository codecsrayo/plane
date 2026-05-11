---
type: community
cohesion: 0.14
members: 15
---

# Webhook Webhookservice

**Cohesion:** 0.14 - loosely connected
**Members:** 15 nodes

## Members

- [[.constructor()_83]] - code - services/webhook.service.ts
- [[.constructor()_16]] - code - store/workspace/webhook.store.ts
- [[.createWebhook()]] - code - services/webhook.service.ts
- [[.currentWebhook()]] - code - store/workspace/webhook.store.ts
- [[.deleteWebhook()]] - code - services/webhook.service.ts
- [[.fetchWebhookDetails()]] - code - services/webhook.service.ts
- [[.fetchWebhooksList()]] - code - services/webhook.service.ts
- [[.regenerateSecretKey()]] - code - services/webhook.service.ts
- [[.updateWebhook()]] - code - services/webhook.service.ts
- [[IWebhookStore]] - code - store/workspace/webhook.store.ts
- [[WebhookService]] - code - services/webhook.service.ts
- [[WebhookStore]] - code - store/workspace/webhook.store.ts
- [[webHookObject]] - code - store/workspace/webhook.store.ts
- [[webhook.service.ts]] - code - services/webhook.service.ts
- [[webhook.store.ts]] - code - store/workspace/webhook.store.ts

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Webhook_Webhookservice
SORT file.name ASC
```

## Connections to other communities

- 3 edges to [[_COMMUNITY_Workspace Invites]]
- 2 edges to [[_COMMUNITY_Webhook Webhooks]]
- 2 edges to [[_COMMUNITY_Project Root Store]]
- 2 edges to [[_COMMUNITY_API Services]]

## Top bridge nodes

- [[webhook.store.ts]] - degree 9, connects to 3 communities
- [[IWebhookStore]] - degree 3, connects to 2 communities
- [[webhook.service.ts]] - degree 4, connects to 1 community
- [[WebhookStore]] - degree 4, connects to 1 community
