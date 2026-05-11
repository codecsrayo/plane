---
type: community
cohesion: 1.00
members: 2
---

# Instance Traces

**Cohesion:** 1.00 - tightly connected
**Members:** 2 nodes

## Members

- [[instance_traces()_1]] - code - api_rust/src/jobs/instance_traces.rs
- [[instance_traces.rs]] - code - api_rust/src/jobs/instance_traces.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Instance_Traces
SORT file.name ASC
```
