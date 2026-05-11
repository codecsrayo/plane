---
type: community
cohesion: 0.18
members: 12
---

# Uuid Test

**Cohesion:** 0.18 - loosely connected
**Members:** 12 nodes

## Members

- [[.add_arguments()_1]] - code - api/plane/db/management/commands/fix_duplicate_sequences.py
- [[.handle()_2]] - code - api/plane/db/management/commands/fix_duplicate_sequences.py
- [[.strict_str_to_int()_1]] - code - api/plane/db/management/commands/fix_duplicate_sequences.py
- [[.test_convert_uuid_to_integer()]] - code - api/plane/tests/unit/utils/test_uuid.py
- [[.test_convert_uuid_to_integer_string_input()]] - code - api/plane/tests/unit/utils/test_uuid.py
- [[Command_3]] - code - api/plane/db/management/commands/fix_duplicate_sequences.py
- [[Convert a UUID to a 64-bit signed integer]] - rationale - api/plane/utils/uuid.py
- [[Test convert_uuid_to_integer function]] - rationale - api/plane/tests/unit/utils/test_uuid.py
- [[Test convert_uuid_to_integer handles string UUID]] - rationale - api/plane/tests/unit/utils/test_uuid.py
- [[convert_uuid_to_integer()]] - code - api/plane/utils/uuid.py
- [[fix_duplicate_sequences.py]] - code - api/plane/db/management/commands/fix_duplicate_sequences.py
- [[uuid.py]] - code - api/plane/utils/uuid.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Uuid_Test
SORT file.name ASC
```

## Connections to other communities

- 3 edges to [[_COMMUNITY_Valid Test]]
- 1 edge to [[_COMMUNITY_Test Validate]]
- 1 edge to [[_COMMUNITY_Command Stripper]]
- 1 edge to [[_COMMUNITY_Command Issue]]

## Top bridge nodes

- [[.test_convert_uuid_to_integer()]] - degree 4, connects to 2 communities
- [[convert_uuid_to_integer()]] - degree 6, connects to 1 community
- [[Command_3]] - degree 5, connects to 1 community
- [[.test_convert_uuid_to_integer_string_input()]] - degree 3, connects to 1 community
- [[uuid.py]] - degree 2, connects to 1 community
