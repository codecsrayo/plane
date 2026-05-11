---
source_file: "api/plane/bgtasks/email_notification_task.py"
type: "code"
community: "Email Process"
location: "L153"
tags:
  - graphify/code
  - graphify/EXTRACTED
  - community/Email_Process
---

# send_email_notification()

## Connections

- [[.set()]] - `calls` [INFERRED]
- [[acquire_lock()]] - `calls` [EXTRACTED]
- [[create_payload()]] - `calls` [EXTRACTED]
- [[email_notification_task.py]] - `contains` [EXTRACTED]
- [[generate_plain_text_from_html()]] - `calls` [INFERRED]
- [[get_email_configuration()]] - `calls` [INFERRED]
- [[log_exception()]] - `calls` [INFERRED]
- [[process_html_content()]] - `calls` [EXTRACTED]
- [[redis_instance()]] - `calls` [INFERRED]
- [[release_lock()]] - `calls` [EXTRACTED]
- [[remove_unwanted_characters()]] - `calls` [EXTRACTED]

#graphify/code #graphify/EXTRACTED #community/Email_Process
