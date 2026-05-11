---
type: community
cohesion: 0.18
members: 19
---

# Community 321

**Cohesion:** 0.18 - loosely connected
**Members:** 19 nodes

## Members
- [[decrypt_config_value()]] - code - api_rust/src/utils/fernet.rs
- [[decrypt_token()]] - code - api_rust/src/utils/token_cipher.rs
- [[derive_fernet_key()]] - code - api_rust/src/utils/fernet.rs
- [[empty_returns_empty()]] - code - api_rust/src/utils/fernet.rs
- [[encrypt_config_value()]] - code - api_rust/src/utils/fernet.rs
- [[encrypt_token()]] - code - api_rust/src/utils/token_cipher.rs
- [[encrypt_without_key_returns_plaintext_with_warning()]] - code - api_rust/src/utils/token_cipher.rs
- [[ensure_configurations_seeded()]] - code - api_rust/src/utils/startup.rs
- [[fernet.rs]] - code - api_rust/src/utils/fernet.rs
- [[fernet_decrypt()]] - code - api_rust/src/utils/fernet.rs
- [[legacy_plaintext_passthrough()]] - code - api_rust/src/utils/token_cipher.rs
- [[load_key()]] - code - api_rust/src/utils/token_cipher.rs
- [[nonces_are_unique_per_call()]] - code - api_rust/src/utils/token_cipher.rs
- [[plaintext_passthrough()]] - code - api_rust/src/utils/fernet.rs
- [[roundtrip_encrypt_decrypt()]] - code - api_rust/src/utils/token_cipher.rs
- [[serialize_config_row()]] - code - api_rust/src/routes/instances.rs
- [[set_test_key()]] - code - api_rust/src/utils/token_cipher.rs
- [[tampered_ciphertext_fails_auth()]] - code - api_rust/src/utils/token_cipher.rs
- [[token_cipher.rs]] - code - api_rust/src/utils/token_cipher.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Community_321
SORT file.name ASC
```

## Connections to other communities
- 5 edges to [[_COMMUNITY_Community 21]]
- 4 edges to [[_COMMUNITY_Rust API Handlers]]
- 1 edge to [[_COMMUNITY_Community 29]]
- 1 edge to [[_COMMUNITY_Community 34]]
- 1 edge to [[_COMMUNITY_Rust Auth Services]]

## Top bridge nodes
- [[ensure_configurations_seeded()]] - degree 6, connects to 4 communities
- [[decrypt_config_value()]] - degree 6, connects to 2 communities
- [[serialize_config_row()]] - degree 3, connects to 2 communities
- [[encrypt_token()]] - degree 8, connects to 1 community
- [[encrypt_config_value()]] - degree 4, connects to 1 community