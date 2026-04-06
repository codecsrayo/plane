"""
GitHub App authentication helpers.

Usage:
    from plane.utils.github_app import get_installation_access_token
    token = get_installation_access_token(installation_id)
    # Use token in Authorization: Bearer {token} header
"""
import base64
import os
import time
from typing import Optional

import requests


def get_installation_access_token(installation_id: str) -> Optional[str]:
    """
    Exchange a GitHub App installation_id for a short-lived installation access token.
    Returns None if GITHUB_APP_ID or GITHUB_APP_PRIVATE_KEY env vars are not set,
    or if the GitHub API call fails.
    """
    app_id = os.environ.get("GITHUB_APP_ID")
    private_key_b64 = os.environ.get("GITHUB_APP_PRIVATE_KEY")

    if not app_id or not private_key_b64:
        return None

    try:
        import jwt
        from cryptography.hazmat.primitives import serialization

        pem = base64.b64decode(private_key_b64)
        private_key = serialization.load_pem_private_key(pem, password=None)

        now = int(time.time())
        payload = {"iat": now - 60, "exp": now + 600, "iss": app_id}
        app_jwt = jwt.encode(payload, private_key, algorithm="RS256")

        resp = requests.post(
            f"https://api.github.com/app/installations/{installation_id}/access_tokens",
            headers={
                "Authorization": f"Bearer {app_jwt}",
                "Accept": "application/vnd.github+json",
                "X-GitHub-Api-Version": "2022-11-28",
            },
            timeout=10,
        )
        if resp.ok:
            return resp.json().get("token")
        return None
    except Exception:
        return None
