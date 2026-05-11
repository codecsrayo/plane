---
source_file: "api_rust/src/auth/forgot_reset_password.rs"
type: "code"
community: "Password Email"
location: "L163"
tags:
  - graphify/code
  - graphify/INFERRED
  - community/Password_Email
---

# handle_forgot_password()

## Connections

- [[.instance_not_configured()]] - `calls` [INFERRED]
- [[.invalid_email()]] - `calls` [INFERRED]
- [[.json()]] - `calls` [INFERRED]
- [[.smtp_not_configured()]] - `calls` [INFERRED]
- [[.to_string()]] - `calls` [INFERRED]
- [[.user_does_not_exist()]] - `calls` [INFERRED]
- [[forgot_password()_1]] - `calls` [EXTRACTED]
- [[forgot_password_space()]] - `calls` [EXTRACTED]
- [[forgot_reset_password.rs]] - `contains` [EXTRACTED]
- [[get_config_value()]] - `calls` [INFERRED]
- [[send_reset_email()]] - `calls` [EXTRACTED]

#graphify/code #graphify/INFERRED #community/Password_Email
