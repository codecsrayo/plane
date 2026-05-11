---
type: community
cohesion: 0.38
members: 11
---

# Extract Comment

**Cohesion:** 0.38 - loosely connected
**Members:** 11 nodes

## Members

- [[TODO Maybe save the comment mentions, so that in future, we can filter out th]] - rationale - api/plane/bgtasks/notification_task.py
- [[create_mention_notification()]] - code - api/plane/bgtasks/notification_task.py
- [[extract_comment_mentions()]] - code - api/plane/bgtasks/notification_task.py
- [[extract_mentions()]] - code - api/plane/bgtasks/notification_task.py
- [[extract_mentions_as_subscribers()]] - code - api/plane/bgtasks/notification_task.py
- [[get_new_comment_mentions()]] - code - api/plane/bgtasks/notification_task.py
- [[get_new_mentions()]] - code - api/plane/bgtasks/notification_task.py
- [[get_removed_mentions()]] - code - api/plane/bgtasks/notification_task.py
- [[notification_task.py]] - code - api/plane/bgtasks/notification_task.py
- [[notifications()]] - code - api/plane/bgtasks/notification_task.py
- [[update_mentions_for_issue()]] - code - api/plane/bgtasks/notification_task.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Extract_Comment
SORT file.name ASC
```

## Connections to other communities

- 3 edges to [[_COMMUNITY_Asset Issue]]
- 2 edges to [[_COMMUNITY_Issue Sync]]
- 1 edge to [[_COMMUNITY_Meta User]]

## Top bridge nodes

- [[notifications()]] - degree 11, connects to 2 communities
- [[extract_mentions()]] - degree 5, connects to 1 community
- [[extract_comment_mentions()]] - degree 4, connects to 1 community
- [[extract_mentions_as_subscribers()]] - degree 3, connects to 1 community
- [[update_mentions_for_issue()]] - degree 3, connects to 1 community
