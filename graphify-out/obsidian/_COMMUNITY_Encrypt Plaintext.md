---
type: community
cohesion: 0.42
members: 10
---

# Encrypt Plaintext

**Cohesion:** 0.42 - moderately connected
**Members:** 10 nodes

## Members

- [[decrypt_token()]] - code - api_rust/src/utils/token_cipher.rs
- [[encrypt_token()]] - code - api_rust/src/utils/token_cipher.rs
- [[encrypt_without_key_returns_plaintext_with_warning()]] - code - api_rust/src/utils/token_cipher.rs
- [[legacy_plaintext_passthrough()]] - code - api_rust/src/utils/token_cipher.rs
- [[load_key()]] - code - api_rust/src/utils/token_cipher.rs
- [[nonces_are_unique_per_call()]] - code - api_rust/src/utils/token_cipher.rs
- [[roundtrip_encrypt_decrypt()]] - code - api_rust/src/utils/token_cipher.rs
- [[set_test_key()]] - code - api_rust/src/utils/token_cipher.rs
- [[tampered_ciphertext_fails_auth()]] - code - api_rust/src/utils/token_cipher.rs
- [[token_cipher.rs]] - code - api_rust/src/utils/token_cipher.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Encrypt_Plaintext
SORT file.name ASC
```

## Connections to other communities

- 2 edges to [[_COMMUNITY_Password Email]]
- 1 edge to [[_COMMUNITY_User Github]]

## Top bridge nodes

- [[encrypt_token()]] - degree 8, connects to 2 communities
- [[decrypt_token()]] - degree 4, connects to 1 community
