# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

import pytest
from django.urls import reverse
from rest_framework import status
from plane.db.models import Importer, WorkspaceMember


@pytest.mark.django_db
class TestImporterAPI:
    def test_github_repositories_list_unauthenticated(self, client, workspace):
        url = reverse("github-repositories", kwargs={"slug": workspace.slug})
        response = client.get(url)
        assert response.status_code == status.HTTP_401_UNAUTHORIZED

    def test_gitlab_repositories_list_unauthenticated(self, client, workspace):
        url = reverse("gitlab-repositories", kwargs={"slug": workspace.slug})
        response = client.get(url)
        assert response.status_code == status.HTTP_401_UNAUTHORIZED

    def test_github_importer_list_authenticated(self, admin_client, workspace):
        url = reverse("github-importer", kwargs={"slug": workspace.slug})
        response = admin_client.get(url)
        assert response.status_code == status.HTTP_200_OK
        assert isinstance(response.json(), list)

    def test_gitlab_importer_list_authenticated(self, admin_client, workspace):
        url = reverse("gitlab-importer", kwargs={"slug": workspace.slug})
        response = admin_client.get(url)
        assert response.status_code == status.HTTP_200_OK
        assert isinstance(response.json(), list)

    def test_github_importer_create_missing_data(self, admin_client, workspace):
        url = reverse("github-importer", kwargs={"slug": workspace.slug})
        response = admin_client.post(url, data={}, content_type="application/json")
        assert response.status_code == status.HTTP_400_BAD_REQUEST

    def test_gitlab_importer_create_missing_data(self, admin_client, workspace):
        url = reverse("gitlab-importer", kwargs={"slug": workspace.slug})
        response = admin_client.post(url, data={}, content_type="application/json")
        assert response.status_code == status.HTTP_400_BAD_REQUEST
