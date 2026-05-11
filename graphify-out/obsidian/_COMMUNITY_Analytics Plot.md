---
type: community
cohesion: 0.60
members: 6
---

# Analytics Plot

**Cohesion:** 0.60 - moderately connected
**Members:** 6 nodes

## Members

- [[analytics_plot.py]] - code - api/plane/utils/analytics_plot.py
- [[annotate_with_monthly_dimension()]] - code - api/plane/utils/analytics_plot.py
- [[build_graph_plot()]] - code - api/plane/utils/analytics_plot.py
- [[burndown_plot()]] - code - api/plane/utils/analytics_plot.py
- [[extract_axis()]] - code - api/plane/utils/analytics_plot.py
- [[sort_data()]] - code - api/plane/utils/analytics_plot.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Analytics_Plot
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Issue Group]]
- 1 edge to [[_COMMUNITY_Endpoint User]]
- 1 edge to [[_COMMUNITY_Generate Analytic]]

## Top bridge nodes

- [[build_graph_plot()]] - degree 7, connects to 3 communities
