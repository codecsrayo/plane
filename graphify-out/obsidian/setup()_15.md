---
source_file: "api_rust/tests/workspace_invitations.rs"
type: "code"
community: "Returns Invitation"
location: "L16"
tags:
  - graphify/code
  - graphify/EXTRACTED
  - community/Returns_Invitation
---

# setup()

## Connections

- [[.spawn()]] - `calls` [INFERRED]
- [[create_invitation_as_admin_returns_2xx()]] - `calls` [EXTRACTED]
- [[create_invitation_unauthenticated_returns_401()]] - `calls` [EXTRACTED]
- [[delete_invitation_nonexistent_returns_4xx()]] - `calls` [EXTRACTED]
- [[delete_invitation_unauthenticated_returns_401()]] - `calls` [EXTRACTED]
- [[get_invitation_nonexistent_returns_404_or_403()]] - `calls` [EXTRACTED]
- [[join_invitation_with_invalid_token_returns_4xx()]] - `calls` [EXTRACTED]
- [[list_invitations_member_returns_200()]] - `calls` [EXTRACTED]
- [[list_invitations_unauthenticated_returns_401()]] - `calls` [EXTRACTED]
- [[update_invitation_nonexistent_returns_4xx()]] - `calls` [EXTRACTED]
- [[workspace_invitations.rs]] - `contains` [EXTRACTED]

#graphify/code #graphify/EXTRACTED #community/Returns_Invitation
