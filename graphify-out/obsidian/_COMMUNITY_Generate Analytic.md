---
type: community
cohesion: 0.22
members: 16
---

# Generate Analytic

**Cohesion:** 0.22 - loosely connected
**Members:** 16 nodes

## Members

- [[Fetch assignee details if required.]] - rationale - api/plane/bgtasks/analytic_plot_export.py
- [[Fetch label details if required]] - rationale - api/plane/bgtasks/analytic_plot_export.py
- [[Generate CSV buffer from rows._1]] - rationale - api/plane/bgtasks/analytic_plot_export.py
- [[Helper function to send export email.]] - rationale - api/plane/bgtasks/analytic_plot_export.py
- [[analytic_export_task()]] - code - api/plane/bgtasks/analytic_plot_export.py
- [[analytic_plot_export.py]] - code - api/plane/bgtasks/analytic_plot_export.py
- [[export_analytics_to_csv_email()]] - code - api/plane/bgtasks/analytic_plot_export.py
- [[generate_csv_from_rows()]] - code - api/plane/bgtasks/analytic_plot_export.py
- [[generate_non_segmented_rows()]] - code - api/plane/bgtasks/analytic_plot_export.py
- [[generate_segmented_rows()]] - code - api/plane/bgtasks/analytic_plot_export.py
- [[get_assignee_details()]] - code - api/plane/bgtasks/analytic_plot_export.py
- [[get_cycle_details()]] - code - api/plane/bgtasks/analytic_plot_export.py
- [[get_label_details()]] - code - api/plane/bgtasks/analytic_plot_export.py
- [[get_module_details()]] - code - api/plane/bgtasks/analytic_plot_export.py
- [[get_state_details()]] - code - api/plane/bgtasks/analytic_plot_export.py
- [[send_export_email()]] - code - api/plane/bgtasks/analytic_plot_export.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Generate_Analytic
SORT file.name ASC
```

## Connections to other communities

- 2 edges to [[_COMMUNITY_Email Task]]
- 2 edges to [[_COMMUNITY_Task Object]]
- 1 edge to [[_COMMUNITY_Exporter Formatter]]
- 1 edge to [[_COMMUNITY_Asset Issue]]
- 1 edge to [[_COMMUNITY_Issue Apply]]
- 1 edge to [[_COMMUNITY_Analytics Plot]]

## Top bridge nodes

- [[analytic_export_task()]] - degree 13, connects to 3 communities
- [[send_export_email()]] - degree 6, connects to 1 community
- [[generate_csv_from_rows()]] - degree 5, connects to 1 community
- [[export_analytics_to_csv_email()]] - degree 4, connects to 1 community
- [[generate_segmented_rows()]] - degree 3, connects to 1 community
