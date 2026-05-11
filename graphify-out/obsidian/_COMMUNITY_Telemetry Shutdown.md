---
type: community
cohesion: 0.33
members: 7
---

# Telemetry Shutdown

**Cohesion:** 0.33 - loosely connected
**Members:** 7 nodes

## Members

- [[Initialize OpenTelemetry with proper shutdown handling]] - rationale - api/plane/utils/telemetry.py
- [[Shutdown OpenTelemetry tracers and processors]] - rationale - api/plane/utils/telemetry.py
- [[init_tracer()]] - code - api/plane/utils/telemetry.py
- [[instance_traces()]] - code - api/plane/license/bgtasks/tracer.py
- [[shutdown_tracer()]] - code - api/plane/utils/telemetry.py
- [[telemetry.py]] - code - api/plane/utils/telemetry.py
- [[tracer.py]] - code - api/plane/license/bgtasks/tracer.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Telemetry_Shutdown
SORT file.name ASC
```
