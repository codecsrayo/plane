---
type: community
cohesion: 0.28
members: 17
---

# Returns Webhook

**Cohesion:** 0.28 - loosely connected
**Members:** 17 nodes

## Members

- [[create_webhook()]] - code - api_rust/tests/webhooks.rs
- [[create_webhook_invalid_url_returns_400()]] - code - api_rust/tests/webhooks.rs
- [[create_webhook_proptest_valid_urls()]] - code - api_rust/tests/webhooks.rs
- [[create_webhook_returns_201()]] - code - api_rust/tests/webhooks.rs
- [[delete_webhook_returns_204()]] - code - api_rust/tests/webhooks.rs
- [[get_deleted_webhook_returns_404()]] - code - api_rust/tests/webhooks.rs
- [[get_webhook_not_found_returns_404()]] - code - api_rust/tests/webhooks.rs
- [[get_webhook_returns_200()]] - code - api_rust/tests/webhooks.rs
- [[list_webhook_logs_returns_200()]] - code - api_rust/tests/webhooks.rs
- [[list_webhook_logs_unauthenticated_returns_401()]] - code - api_rust/tests/webhooks.rs
- [[list_webhooks_empty_returns_200()]] - code - api_rust/tests/webhooks.rs
- [[list_webhooks_returns_created_webhook()]] - code - api_rust/tests/webhooks.rs
- [[list_webhooks_unauthenticated_returns_401()]] - code - api_rust/tests/webhooks.rs
- [[regenerate_webhook_secret_returns_200()]] - code - api_rust/tests/webhooks.rs
- [[setup()_34]] - code - api_rust/tests/webhooks.rs
- [[update_webhook_returns_200()]] - code - api_rust/tests/webhooks.rs
- [[webhooks.rs]] - code - api_rust/tests/webhooks.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Returns_Webhook
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Returns Sign]]
- 1 edge to [[_COMMUNITY_Password Email]]

## Top bridge nodes

- [[setup()_34]] - degree 16, connects to 1 community
- [[create_webhook_proptest_valid_urls()]] - degree 3, connects to 1 community
