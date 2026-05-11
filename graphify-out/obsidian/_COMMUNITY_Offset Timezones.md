---
type: community
cohesion: 0.31
members: 10
---

# Offset Timezones

**Cohesion:** 0.31 - loosely connected
**Members:** 10 nodes

## Members

- [[TimezoneEntry]] - code - api_rust/src/routes/timezones.rs
- [[TimezonesResponse]] - code - api_rust/src/routes/timezones.rs
- [[bogota_entry_has_expected_offset()]] - code - api_rust/src/routes/timezones.rs
- [[build_timezone_list()]] - code - api_rust/src/routes/timezones.rs
- [[duplicated_india_entries_all_present()]] - code - api_rust/src/routes/timezones.rs
- [[format_offset_hhmm()]] - code - api_rust/src/routes/timezones.rs
- [[format_offset_positive_zero_and_negative()]] - code - api_rust/src/routes/timezones.rs
- [[list_sorted_by_offset_then_label()]] - code - api_rust/src/routes/timezones.rs
- [[list_timezones()]] - code - api_rust/src/routes/timezones.rs
- [[timezones.rs]] - code - api_rust/src/routes/timezones.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Offset_Timezones
SORT file.name ASC
```

## Connections to other communities

- 1 edge to [[_COMMUNITY_Issue Request]]
- 1 edge to [[_COMMUNITY_User Github]]

## Top bridge nodes

- [[build_timezone_list()]] - degree 7, connects to 1 community
- [[list_timezones()]] - degree 3, connects to 1 community
