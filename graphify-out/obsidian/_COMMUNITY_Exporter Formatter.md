---
type: community
cohesion: 0.04
members: 62
---

# Exporter Formatter

**Cohesion:** 0.04 - loosely connected
**Members:** 62 nodes

## Members

- [[.__init__()_19]] - code - api/plane/utils/exporters/exporter.py
- [[.__init__()_24]] - code - api/plane/utils/porters/exporter.py
- [[.__init__()_22]] - code - api/plane/utils/porters/formatters.py
- [[.__init__()_21]] - code - api/plane/utils/porters/formatters.py
- [[.__init__()_23]] - code - api/plane/utils/porters/formatters.py
- [[._create_formatter()]] - code - api/plane/utils/porters/exporter.py
- [[._flatten()]] - code - api/plane/utils/porters/formatters.py
- [[._format_value()_8]] - code - api/plane/utils/porters/formatters.py
- [[._normalize_header()]] - code - api/plane/utils/porters/formatters.py
- [[._normalize_header()_1]] - code - api/plane/utils/porters/formatters.py
- [[._prettify_header()]] - code - api/plane/utils/porters/formatters.py
- [[._prettify_header()_1]] - code - api/plane/utils/porters/formatters.py
- [[._unflatten()]] - code - api/plane/utils/porters/formatters.py
- [[.decode()_1]] - code - api/plane/utils/porters/formatters.py
- [[.decode()]] - code - api/plane/utils/porters/formatters.py
- [[.decode()_2]] - code - api/plane/utils/porters/formatters.py
- [[.encode()_1]] - code - api/plane/utils/porters/formatters.py
- [[.encode()]] - code - api/plane/utils/porters/formatters.py
- [[.encode()_2]] - code - api/plane/utils/porters/formatters.py
- [[.export()]] - code - api/plane/utils/exporters/exporter.py
- [[.export()_1]] - code - api/plane/utils/porters/exporter.py
- [[.serialize()_1]] - code - api/plane/utils/porters/exporter.py
- [[.to_file()]] - code - api/plane/utils/porters/exporter.py
- [[ABC]] - code
- [[Args             flatten Whether to flatten nested dicts.             delimite]] - rationale - api/plane/utils/porters/formatters.py
- [[Args             prettify_headers If True, transforms 'created_by_name' → 'Cre]] - rationale - api/plane/utils/porters/formatters.py
- [[BaseFormatter_1]] - code - api/plane/utils/porters/formatters.py
- [[CSVFormatter_1]] - code - api/plane/utils/porters/formatters.py
- [[Create formatter instance with appropriate options.]] - rationale - api/plane/utils/porters/exporter.py
- [[DataExporter]] - code - api/plane/utils/porters/exporter.py
- [[Decode CSV content to list of dicts.          Args             content CSV str]] - rationale - api/plane/utils/porters/formatters.py
- [[Decode XLSX bytes to list of dicts.          Args             content XLSX fil]] - rationale - api/plane/utils/porters/formatters.py
- [[Encode data to XLSX bytes.]] - rationale - api/plane/utils/porters/formatters.py
- [[Export data using DRF serializers with built-in format support.      Usage]] - rationale - api/plane/utils/porters/exporter.py
- [[Export data using the configured formatter and return (filename, content).]] - rationale - api/plane/utils/exporters/exporter.py
- [[Export queryset to file with configured format.          Args             filen]] - rationale - api/plane/utils/porters/exporter.py
- [[Export to file (legacy interface)]] - rationale - api/plane/utils/porters/exporter.py
- [[Exporter_1]] - code - api/plane/utils/exporters/exporter.py
- [[Format a value for XLSX cell.]] - rationale - api/plane/utils/porters/formatters.py
- [[Formatter for XLSX (Excel) files using openpyxl.]] - rationale - api/plane/utils/porters/formatters.py
- [[Generic exporter class that handles data exports using different formatters.]] - rationale - api/plane/utils/exporters/exporter.py
- [[Initialize exporter with serializer and optional format type.          Args]] - rationale - api/plane/utils/porters/exporter.py
- [[Initialize exporter with specified format type and schema.          Args]] - rationale - api/plane/utils/exporters/exporter.py
- [[JSONFormatter_1]] - code - api/plane/utils/porters/formatters.py
- [[QuerySet → list of dicts]] - rationale - api/plane/utils/porters/exporter.py
- [[Sanitize a value for CSV export to prevent formula injection.      Prefixes stri]] - rationale - api/plane/utils/csv_utils.py
- [[Sanitize all values in a CSV row.]] - rationale - api/plane/utils/csv_utils.py
- [[Transform 'Display Name' → 'display_name' (reverse of prettify)_1]] - rationale - api/plane/utils/porters/formatters.py
- [[Transform 'Display Name' → 'display_name' (reverse of prettify)]] - rationale - api/plane/utils/porters/formatters.py
- [[Transform 'created_by_name' → 'Created By Name_1]] - rationale - api/plane/utils/porters/formatters.py
- [[Transform 'created_by_name' → 'Created By Name]] - rationale - api/plane/utils/porters/formatters.py
- [[XLSXFormatter_1]] - code - api/plane/utils/porters/formatters.py
- [[__init__.py_40]] - code - api/plane/utils/porters/**init**.py
- [[csv_utils.py]] - code - api/plane/utils/csv_utils.py
- [[decode()]] - code - api/plane/utils/porters/formatters.py
- [[encode()]] - code - api/plane/utils/porters/formatters.py
- [[exporter.py_4]] - code - api/plane/utils/porters/exporter.py
- [[extension()]] - code - api/plane/utils/porters/formatters.py
- [[formatters.py_1]] - code - api/plane/utils/porters/formatters.py
- [[get_available_formats()_1]] - code - api/plane/utils/porters/exporter.py
- [[sanitize_csv_row()]] - code - api/plane/utils/csv_utils.py
- [[sanitize_csv_value()]] - code - api/plane/utils/csv_utils.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Exporter_Formatter
SORT file.name ASC
```

## Connections to other communities

- 6 edges to [[_COMMUNITY_Test Validate]]
- 3 edges to [[_COMMUNITY_Session Presigned]]
- 2 edges to [[_COMMUNITY_Formatter Generate]]
- 1 edge to [[_COMMUNITY_Endpoint User]]
- 1 edge to [[_COMMUNITY_Generate Analytic]]
- 1 edge to [[_COMMUNITY_Provider Upload]]

## Top bridge nodes

- [[sanitize_csv_row()]] - degree 7, connects to 3 communities
- [[DataExporter]] - degree 13, connects to 2 communities
- [[Exporter_1]] - degree 7, connects to 1 community
- [[sanitize_csv_value()]] - degree 5, connects to 1 community
- [[.serialize()_1]] - degree 4, connects to 1 community
