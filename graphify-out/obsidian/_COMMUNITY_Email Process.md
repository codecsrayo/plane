---
type: community
cohesion: 0.39
members: 9
---

# Email Process

**Cohesion:** 0.39 - loosely connected
**Members:** 9 nodes

## Members

- [[acquire_lock()]] - code - api/plane/bgtasks/email_notification_task.py
- [[create_payload()]] - code - api/plane/bgtasks/email_notification_task.py
- [[email_notification_task.py]] - code - api/plane/bgtasks/email_notification_task.py
- [[process_html_content()]] - code - api/plane/bgtasks/email_notification_task.py
- [[process_mention()]] - code - api/plane/bgtasks/email_notification_task.py
- [[release_lock()]] - code - api/plane/bgtasks/email_notification_task.py
- [[remove_unwanted_characters()]] - code - api/plane/bgtasks/email_notification_task.py
- [[send_email_notification()]] - code - api/plane/bgtasks/email_notification_task.py
- [[stack_email_notification()]] - code - api/plane/bgtasks/email_notification_task.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Email_Process
SORT file.name ASC
```

## Connections to other communities

- 3 edges to [[_COMMUNITY_Test Magic]]
- 2 edges to [[_COMMUNITY_Asset Issue]]
- 2 edges to [[_COMMUNITY_Email Task]]
- 1 edge to [[_COMMUNITY_Task Object]]

## Top bridge nodes

- [[send_email_notification()]] - degree 11, connects to 4 communities
- [[acquire_lock()]] - degree 3, connects to 1 community
- [[release_lock()]] - degree 3, connects to 1 community
- [[stack_email_notification()]] - degree 2, connects to 1 community
