---
source_file: "api/plane/bgtasks/copy_s3_object.py"
type: "code"
community: "Copy Test"
location: "L125"
tags:
  - graphify/code
  - graphify/EXTRACTED
  - community/Copy_Test
---

# copy_s3_objects_of_description_and_assets()

## Connections

- [[Step 1 Extract asset ids from the description_html of the entity     Step 2 Du]] - `rationale_for` [EXTRACTED]
- [[copy_assets()]] - `calls` [EXTRACTED]
- [[copy_s3_object.py]] - `contains` [EXTRACTED]
- [[extract_asset_ids()]] - `calls` [EXTRACTED]
- [[log_exception()]] - `calls` [INFERRED]
- [[sync_with_external_service()]] - `calls` [EXTRACTED]
- [[test_copy_s3_objects_of_description_and_assets()]] - `calls` [INFERRED]
- [[update_description()]] - `calls` [EXTRACTED]

#graphify/code #graphify/EXTRACTED #community/Copy_Test
