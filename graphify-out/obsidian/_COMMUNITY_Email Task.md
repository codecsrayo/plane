---
type: community
cohesion: 0.11
members: 25
---

# Email Task

**Cohesion:** 0.11 - loosely connected
**Members:** 25 nodes

## Members

- [[Generate clean plain text from HTML email template.     Removes all HTML tags, C]] - rationale - api/plane/utils/email.py
- [[Send a confirmation email to the user after their email address has been success]] - rationale - api/plane/bgtasks/user_email_update_task.py
- [[Send an email notification when a webhook is deactivated.      Args         web]] - rationale - api/plane/bgtasks/webhook_task.py
- [[email.py]] - code - api/plane/utils/email.py
- [[forgot_password()]] - code - api/plane/bgtasks/forgot_password_task.py
- [[forgot_password_task.py]] - code - api/plane/bgtasks/forgot_password_task.py
- [[generate_plain_text_from_html()]] - code - api/plane/utils/email.py
- [[get_email_configuration()]] - code - api/plane/license/utils/instance_value.py
- [[instance_value.py]] - code - api/plane/license/utils/instance_value.py
- [[magic_link()]] - code - api/plane/bgtasks/magic_link_code_task.py
- [[magic_link_code_task.py]] - code - api/plane/bgtasks/magic_link_code_task.py
- [[project_add_user_email()]] - code - api/plane/bgtasks/project_add_user_email_task.py
- [[project_add_user_email_task.py]] - code - api/plane/bgtasks/project_add_user_email_task.py
- [[project_invitation()]] - code - api/plane/bgtasks/project_invitation_task.py
- [[project_invitation_task.py]] - code - api/plane/bgtasks/project_invitation_task.py
- [[send_email_update_confirmation()]] - code - api/plane/bgtasks/user_email_update_task.py
- [[send_email_update_magic_code()]] - code - api/plane/bgtasks/user_email_update_task.py
- [[send_webhook_deactivation_email()]] - code - api/plane/bgtasks/webhook_task.py
- [[user_activation_email()]] - code - api/plane/bgtasks/user_activation_email_task.py
- [[user_activation_email_task.py]] - code - api/plane/bgtasks/user_activation_email_task.py
- [[user_deactivation_email()]] - code - api/plane/bgtasks/user_deactivation_email_task.py
- [[user_deactivation_email_task.py]] - code - api/plane/bgtasks/user_deactivation_email_task.py
- [[user_email_update_task.py]] - code - api/plane/bgtasks/user_email_update_task.py
- [[workspace_invitation()]] - code - api/plane/bgtasks/workspace_invitation_task.py
- [[workspace_invitation_task.py]] - code - api/plane/bgtasks/workspace_invitation_task.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Email_Task
SORT file.name ASC
```

## Connections to other communities

- 10 edges to [[_COMMUNITY_Task Object]]
- 2 edges to [[_COMMUNITY_Email Process]]
- 2 edges to [[_COMMUNITY_Generate Analytic]]
- 2 edges to [[_COMMUNITY_Command Stripper]]
- 2 edges to [[_COMMUNITY_Endpoint Magic]]
- 1 edge to [[_COMMUNITY_Webhook Model]]
- 1 edge to [[_COMMUNITY_Instance Endpoint]]

## Top bridge nodes

- [[get_email_configuration()]] - degree 16, connects to 5 communities
- [[generate_plain_text_from_html()]] - degree 15, connects to 3 communities
- [[send_webhook_deactivation_email()]] - degree 5, connects to 2 communities
- [[send_email_update_confirmation()]] - degree 5, connects to 1 community
- [[forgot_password()]] - degree 4, connects to 1 community
