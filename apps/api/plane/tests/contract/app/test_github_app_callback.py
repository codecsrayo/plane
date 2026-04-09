# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.
"""
Contract tests for GithubAppCallbackEndpoint.

Covers the critical bug where actor=None caused a silent DB save failure
that previously returned success=True without persisting the installation_id.
"""

import pytest
from django.urls import reverse

from plane.db.models import Integration, Workspace, WorkspaceIntegration, WorkspaceMember


@pytest.fixture
def github_integration(db):
    """Ensure the github Integration record exists."""
    integration, _ = Integration.objects.get_or_create(
        provider="github",
        defaults={
            "title": "GitHub",
            "network": 2,
            "verified": True,
        },
    )
    return integration


@pytest.fixture
def workspace_with_admin(create_user, db):
    """Create a workspace and make create_user an admin member."""
    ws = Workspace.objects.create(
        name="Callback Test WS",
        slug="callback-test-ws",
        owner=create_user,
    )
    WorkspaceMember.objects.get_or_create(
        workspace=ws,
        member=create_user,
        defaults={"role": 20, "is_active": True},
    )
    return ws, create_user


@pytest.fixture
def workspace_no_admin(create_user, db):
    """Create a workspace with NO admin members (deactivated/removed)."""
    ws = Workspace.objects.create(
        name="No Admin WS",
        slug="no-admin-ws",
        owner=create_user,
    )
    # Mark the member as inactive so role>=20 filter returns nothing
    WorkspaceMember.objects.filter(workspace=ws).update(is_active=False)
    return ws


@pytest.mark.contract
class TestGithubAppCallbackEndpoint:
    """Tests for the unauthenticated GitHub App installation callback."""

    @pytest.mark.django_db
    def test_saves_installation_id_to_db(self, client, workspace_with_admin, github_integration):
        """Happy path: installation_id is persisted when workspace has an admin."""
        ws, _ = workspace_with_admin
        url = reverse("github-app-callback")

        response = client.get(
            url,
            {
                "installation_id": "99887766",
                "setup_action": "install",
                "state": ws.slug,
            },
        )

        assert response.status_code == 200
        assert b"success" in response.content
        assert b'"success": true' in response.content

        wi = WorkspaceIntegration.objects.filter(
            workspace=ws,
            integration=github_integration,
        ).first()
        assert wi is not None, "WorkspaceIntegration must be created"
        assert wi.metadata.get("installation_id") == "99887766"
        assert wi.config.get("installation_id") == "99887766"

    @pytest.mark.django_db
    def test_updates_installation_id_on_reinstall(self, client, workspace_with_admin, github_integration):
        """Re-install: existing WorkspaceIntegration metadata/config gets updated."""
        ws, user = workspace_with_admin
        url = reverse("github-app-callback")

        # First install
        client.get(url, {"installation_id": "11111111", "setup_action": "install", "state": ws.slug})

        # Re-install with new installation_id
        response = client.get(url, {"installation_id": "22222222", "setup_action": "update", "state": ws.slug})

        assert response.status_code == 200
        assert b'"success": true' in response.content

        wi = WorkspaceIntegration.objects.get(workspace=ws, integration=github_integration)
        assert wi.metadata.get("installation_id") == "22222222", "installation_id must be updated on re-install"
        assert wi.config.get("installation_id") == "22222222"

    @pytest.mark.django_db
    def test_returns_error_when_no_admin_found(self, client, workspace_no_admin, github_integration):
        """
        Bug fix: when no active admin exists, must return success=false
        and NOT silently discard the installation_id with a false success.
        """
        ws = workspace_no_admin
        url = reverse("github-app-callback")

        response = client.get(
            url,
            {"installation_id": "55555555", "setup_action": "install", "state": ws.slug},
        )

        assert response.status_code == 200
        assert b'"success": false' in response.content

        # Critically: nothing was saved to DB
        assert not WorkspaceIntegration.objects.filter(
            workspace=ws,
            integration=github_integration,
        ).exists(), "No WorkspaceIntegration should be created when actor cannot be determined"

    @pytest.mark.django_db
    def test_returns_error_for_missing_installation_id(self, client, workspace_with_admin, github_integration):
        """Missing installation_id param must return failure, not 500."""
        ws, _ = workspace_with_admin
        url = reverse("github-app-callback")

        response = client.get(url, {"state": ws.slug})

        assert response.status_code == 200
        assert b'"success": false' in response.content

    @pytest.mark.django_db
    def test_returns_error_for_unknown_workspace(self, client, github_integration):
        """Unknown workspace slug must return failure, not 500."""
        url = reverse("github-app-callback")

        response = client.get(
            url,
            {"installation_id": "12345", "state": "nonexistent-workspace-slug"},
        )

        assert response.status_code == 200
        assert b'"success": false' in response.content
