---
type: community
cohesion: 0.83
members: 4
---

# Event Tracking

**Cohesion:** 0.83 - tightly connected
**Members:** 4 nodes

## Members

- [[event_tracking_task.py]] - code - api/plane/bgtasks/event_tracking_task.py
- [[posthogConfiguration()]] - code - api/plane/bgtasks/event_tracking_task.py
- [[preprocess_data_properties()]] - code - api/plane/bgtasks/event_tracking_task.py
- [[track_event()]] - code - api/plane/bgtasks/event_tracking_task.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Event_Tracking
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Endpoint Magic]]
- 1 edge to [[_COMMUNITY_Task Object]]

## Top bridge nodes

- [[track_event()]] - degree 4, connects to 1 community
- [[posthogConfiguration()]] - degree 3, connects to 1 community
