---
type: community
cohesion: 0.23
members: 15
---

# Returns Notification

**Cohesion:** 0.23 - loosely connected
**Members:** 15 nodes

## Members

- [[archive_notification_returns_200()]] - code - api_rust/tests/notifications.rs
- [[delete_notification_returns_204()]] - code - api_rust/tests/notifications.rs
- [[get_notification_by_id_returns_200()]] - code - api_rust/tests/notifications.rs
- [[get_notification_not_found_returns_404()]] - code - api_rust/tests/notifications.rs
- [[get_notification_preferences_returns_200()]] - code - api_rust/tests/notifications.rs
- [[list_notifications_empty_returns_200()]] - code - api_rust/tests/notifications.rs
- [[list_notifications_unauthenticated_returns_401()]] - code - api_rust/tests/notifications.rs
- [[list_notifications_with_filters_returns_200()]] - code - api_rust/tests/notifications.rs
- [[mark_all_read_returns_200()]] - code - api_rust/tests/notifications.rs
- [[mark_notification_read_returns_200()]] - code - api_rust/tests/notifications.rs
- [[notifications.rs]] - code - api_rust/tests/notifications.rs
- [[seed_notification()]] - code - api_rust/tests/notifications.rs
- [[setup()_7]] - code - api_rust/tests/notifications.rs
- [[unread_count_returns_200_with_counts()]] - code - api_rust/tests/notifications.rs
- [[update_notification_preferences_returns_200()]] - code - api_rust/tests/notifications.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Returns_Notification
SORT file.name ASC
```

## Connections to other communities

- 7 edges to [[_COMMUNITY_Returns Sign]]
- 1 edge to [[_COMMUNITY_Issue Request]]
- 1 edge to [[_COMMUNITY_Asset Issue]]
- 1 edge to [[_COMMUNITY_Password Email]]

## Top bridge nodes

- [[seed_notification()]] - degree 8, connects to 3 communities
- [[setup()_7]] - degree 8, connects to 1 community
- [[archive_notification_returns_200()]] - degree 3, connects to 1 community
- [[delete_notification_returns_204()]] - degree 3, connects to 1 community
- [[get_notification_by_id_returns_200()]] - degree 3, connects to 1 community
