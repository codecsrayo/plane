# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

from unittest.mock import MagicMock, patch

import pytest
from django.urls import reverse
from rest_framework import status

from plane.db.models import UserGithubConnection


@pytest.mark.contract
class TestGithubUserConnectionEndpoint:
    @pytest.mark.django_db
    def test_connect_github_account_success(self, session_client, create_user):
        session_client.force_authenticate(user=create_user)
        url = reverse("github-user-callback")

        token_response = MagicMock()
        token_response.json.return_value = {"access_token": "github-user-token"}
        token_response.raise_for_status.return_value = None

        user_response = MagicMock()
        user_response.json.return_value = {
            "id": 12345,
            "login": "plane-user",
            "avatar_url": "https://avatars.githubusercontent.com/u/12345",
        }
        user_response.raise_for_status.return_value = None

        with (
            patch("plane.app.views.integration.base.get_configuration_value", return_value=("client-id", "client-secret")),
            patch("plane.app.views.integration.base.base_host", return_value="http://localhost:3000"),
            patch("plane.app.views.integration.base.requests.post", return_value=token_response) as mock_post,
            patch("plane.app.views.integration.base.requests.get", return_value=user_response) as mock_get,
        ):
            response = session_client.post(url, {"code": "oauth-code"}, format="json")

        assert response.status_code == status.HTTP_201_CREATED
        assert response.data["github_user_id"] == "12345"
        assert response.data["github_username"] == "plane-user"
        assert response.data["created"] is True

        connection = UserGithubConnection.objects.get(user=create_user)
        assert connection.access_token == "github-user-token"
        assert connection.github_username == "plane-user"

        mock_post.assert_called_once_with(
            "https://github.com/login/oauth/access_token",
            data={
                "client_id": "client-id",
                "client_secret": "client-secret",
                "code": "oauth-code",
                "redirect_uri": "http://localhost:3000/auth/github/user-callback/",
            },
            headers={"Accept": "application/json"},
            timeout=10,
        )
        mock_get.assert_called_once_with(
            "https://api.github.com/user",
            headers={
                "Authorization": "Bearer github-user-token",
                "Accept": "application/json",
            },
            timeout=10,
        )

    @pytest.mark.django_db
    def test_connect_github_account_requires_code(self, session_client, create_user):
        session_client.force_authenticate(user=create_user)
        url = reverse("github-user-callback")

        response = session_client.post(url, {}, format="json")

        assert response.status_code == status.HTTP_400_BAD_REQUEST
        assert response.data["error"] == "code is required"
        assert UserGithubConnection.objects.filter(user=create_user).exists() is False
