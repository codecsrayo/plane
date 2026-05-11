---
source_file: "api/plane/authentication/rate_limit.py"
type: "code"
community: "Community 53"
location: "L31"
tags:
  - graphify/code
  - graphify/INFERRED
  - community/Community_53
---

# EmailVerificationThrottle

## Connections
- [[.get_throttles()]] - `calls` [INFERRED]
- [[.throttle_failure_view()_1]] - `method` [EXTRACTED]
- [[AccountEndpoint]] - `uses` [INFERRED]
- [[AuthenticationException]] - `uses` [INFERRED]
- [[ProfileEndpoint]] - `uses` [INFERRED]
- [[Throttle for email verification code generation.     Limits to 3 requests per ho]] - `rationale_for` [EXTRACTED]
- [[UpdateUserOnBoardedEndpoint]] - `uses` [INFERRED]
- [[UpdateUserTourCompletedEndpoint]] - `uses` [INFERRED]
- [[UserActivityEndpoint]] - `uses` [INFERRED]
- [[UserEndpoint]] - `uses` [INFERRED]
- [[UserRateThrottle]] - `inherits` [EXTRACTED]
- [[UserSessionEndpoint]] - `uses` [INFERRED]
- [[rate_limit.py_1]] - `contains` [EXTRACTED]

#graphify/code #graphify/INFERRED #community/Community_53