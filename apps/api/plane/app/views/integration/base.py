# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

# Python imports
import os
import requests

# Third party imports
from django.shortcuts import get_object_or_404
from rest_framework import status
from rest_framework.response import Response
from rest_framework.permissions import IsAuthenticated

# Module imports
from plane.app.views.base import BaseViewSet, BaseAPIView
from plane.db.models import (
    Integration,
    WorkspaceIntegration,
    Workspace,
    APIToken,
    GithubRepository,
    GithubRepositorySync,
)
from plane.app.serializers import IntegrationSerializer, WorkspaceIntegrationSerializer
from plane.app.permissions import WorkSpaceAdminPermission, ROLE, allow_permission
from plane.license.utils.instance_value import get_configuration_value


class IntegrationViewSet(BaseViewSet):
    """
    Viewset to list all available integrations.
    """

    model = Integration
    serializer_class = IntegrationSerializer
    permission_classes = [
        IsAuthenticated,
    ]

    def get_queryset(self):
        # List all integrations, not just verified ones, to ensure they appear in the panel
        return self.model.objects.all()


class WorkspaceIntegrationViewSet(BaseViewSet):
    """
    Viewset to manage workspace level integrations.
    """

    model = WorkspaceIntegration
    serializer_class = WorkspaceIntegrationSerializer
    permission_classes = [
        WorkSpaceAdminPermission,
    ]

    def get_queryset(self):
        return self.model.objects.filter(workspace__slug=self.kwargs.get("slug")).select_related(
            "integration", "workspace", "actor"
        )

    @allow_permission([ROLE.ADMIN], level="WORKSPACE")
    def create(self, request, slug):
        """
        Install an integration in the workspace.
        """
        integration_id = request.data.get("integration")
        if not integration_id:
            return Response(
                {"error": "Integration ID is required"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        workspace = get_object_or_404(Workspace, slug=slug)
        integration = get_object_or_404(Integration, pk=integration_id)

        # Create a Plane API token for the integration if it doesn't exist
        api_token, _ = APIToken.objects.get_or_create(
            user=request.user,
            workspace=workspace,
            defaults={"label": f"{integration.title} Integration Token"},
        )

        workspace_integration, created = WorkspaceIntegration.objects.get_or_create(
            workspace=workspace,
            integration=integration,
            defaults={
                "actor": request.user,
                "api_token": api_token,
                "metadata": request.data.get("metadata", {}),
                "config": request.data.get("config", {}),
            },
        )

        if not created:
            return Response(
                {"error": "Integration already installed"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        serializer = WorkspaceIntegrationSerializer(workspace_integration)
        return Response(serializer.data, status=status.HTTP_201_CREATED)

    @allow_permission([ROLE.ADMIN], level="WORKSPACE")
    def destroy(self, request, slug, pk):
        """
        Uninstall an integration from the workspace.
        """
        workspace_integration = get_object_or_404(
            WorkspaceIntegration,
            pk=pk,
            workspace__slug=slug
        )
        workspace_integration.delete()
        return Response(status=status.HTTP_204_NO_CONTENT)

    @allow_permission([ROLE.ADMIN], level="WORKSPACE")
    def provider_destroy(self, request, slug, provider):
        """
        Uninstall an integration from the workspace using the provider name.
        """
        workspace_integration = get_object_or_404(
            WorkspaceIntegration,
            integration__provider=provider,
            workspace__slug=slug
        )
        workspace_integration.delete()
        return Response(status=status.HTTP_204_NO_CONTENT)

    @allow_permission([ROLE.ADMIN], level="WORKSPACE")
    def provider_install(self, request, slug, provider):
        """
        Install an integration for a given provider (github, gitlab, slack) using
        the OAuth callback data (installation_id, code, etc.) received after the
        user completes the OAuth flow in the popup window.

        Expected payload varies by provider:
          - GitHub:  { "installation_id": "12345678" }
          - GitLab:  { "code": "xxxx" }
          - Slack:   { "code": "xxxx" }
        """
        workspace = get_object_or_404(Workspace, slug=slug)
        integration = get_object_or_404(Integration, provider=provider)

        # Build metadata from the OAuth callback data
        metadata = {}
        config = {}

        if provider == "github":
            installation_id = request.data.get("installation_id")
            if not installation_id:
                return Response(
                    {"error": "installation_id is required for GitHub integration"},
                    status=status.HTTP_400_BAD_REQUEST,
                )

            # Store the installation_id so the workspace is linked to the GitHub App install.
            # Fetching an installation access token requires a signed App JWT (RS256 private key),
            # which is not yet configured.  The installation_id is sufficient to identify the
            # install and can be used to generate tokens later once the private key is added.
            metadata = {"installation_id": installation_id}
            config = {"installation_id": installation_id}

        elif provider == "gitlab":
            code = request.data.get("code")
            if not code:
                return Response(
                    {"error": "code is required for GitLab integration"},
                    status=status.HTTP_400_BAD_REQUEST,
                )
            metadata = {"code": code}

        elif provider == "slack":
            code = request.data.get("code")
            if not code:
                return Response(
                    {"error": "code is required for Slack integration"},
                    status=status.HTTP_400_BAD_REQUEST,
                )

            (SLACK_CLIENT_ID, SLACK_CLIENT_SECRET) = get_configuration_value(
                [
                    {"key": "SLACK_CLIENT_ID", "default": os.environ.get("SLACK_CLIENT_ID", "")},
                    {"key": "SLACK_CLIENT_SECRET", "default": os.environ.get("SLACK_CLIENT_SECRET", "")},
                ]
            )

            if SLACK_CLIENT_ID and SLACK_CLIENT_SECRET:
                try:
                    slack_response = requests.post(
                        "https://slack.com/api/oauth.v2.access",
                        data={
                            "client_id": SLACK_CLIENT_ID,
                            "client_secret": SLACK_CLIENT_SECRET,
                            "code": code,
                        },
                        timeout=10,
                    )
                    if slack_response.ok:
                        slack_data = slack_response.json()
                        metadata = slack_data
                        config = {
                            "access_token": slack_data.get("access_token"),
                            "team_id": slack_data.get("team", {}).get("id"),
                            "team_name": slack_data.get("team", {}).get("name"),
                        }
                except Exception:
                    metadata = {"code": code}
            else:
                metadata = {"code": code}

        # Get or create an API token for this workspace
        api_token, _ = APIToken.objects.get_or_create(
            user=request.user,
            workspace=workspace,
            defaults={"label": f"{integration.title} Integration Token"},
        )

        # Create (or update if already exists) the WorkspaceIntegration
        workspace_integration, created = WorkspaceIntegration.objects.get_or_create(
            workspace=workspace,
            integration=integration,
            defaults={
                "actor": request.user,
                "api_token": api_token,
                "metadata": metadata,
                "config": config,
            },
        )

        if not created:
            # Update metadata/config with fresh OAuth data in case of re-install
            workspace_integration.metadata = metadata
            workspace_integration.config = config
            workspace_integration.save(update_fields=["metadata", "config"])

        serializer = WorkspaceIntegrationSerializer(workspace_integration)
        return Response(serializer.data, status=status.HTTP_201_CREATED if created else status.HTTP_200_OK)


class GithubRepoSyncViewSet(BaseViewSet):
    """
    Viewset to manage GithubRepositorySync records for a workspace's GitHub integration.
    """

    model = GithubRepositorySync
    permission_classes = [WorkSpaceAdminPermission]

    def get_queryset(self):
        workspace_integration = WorkspaceIntegration.objects.filter(
            workspace__slug=self.kwargs.get("slug"),
            integration__provider="github",
        ).first()
        if not workspace_integration:
            return self.model.objects.none()
        return self.model.objects.filter(
            workspace_integration=workspace_integration,
        ).select_related("repository", "workspace_integration", "actor")

    @allow_permission([ROLE.ADMIN], level="WORKSPACE")
    def list(self, request, slug):
        """Return all repo syncs for this workspace's GitHub integration."""
        syncs = self.get_queryset()
        data = [
            {
                "id": str(s.id),
                "project_id": str(s.project_id),
                "repo_id": str(s.repository.repository_id),
                "repo_full_name": f"{s.repository.owner}/{s.repository.name}",
                "repo_name": s.repository.name,
                "repo_owner": s.repository.owner,
                "created_at": s.created_at,
            }
            for s in syncs
        ]
        return Response(data, status=status.HTTP_200_OK)

    @allow_permission([ROLE.ADMIN], level="WORKSPACE")
    def create(self, request, slug):
        """
        Create a GithubRepositorySync linking a project with a GitHub repo.
        Body: { repo_id, repo_full_name, project_id }
        """
        repo_id = request.data.get("repo_id")
        repo_full_name = request.data.get("repo_full_name", "")
        project_id = request.data.get("project_id")

        if not all([repo_id, project_id]):
            return Response(
                {"error": "repo_id and project_id are required"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        workspace = get_object_or_404(Workspace, slug=slug)

        workspace_integration = WorkspaceIntegration.objects.filter(
            workspace=workspace,
            integration__provider="github",
        ).first()
        if not workspace_integration:
            return Response(
                {"error": "GitHub integration not installed for this workspace"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        # Parse owner / name from full_name (e.g. "owner/repo")
        parts = repo_full_name.split("/", 1)
        repo_owner = parts[0] if len(parts) == 2 else ""
        repo_name = parts[1] if len(parts) == 2 else repo_full_name

        # Get or create the GithubRepository record
        repo, _ = GithubRepository.objects.get_or_create(
            repository_id=int(repo_id),
            project_id=project_id,
            workspace=workspace,
            defaults={
                "name": repo_name,
                "owner": repo_owner,
                "url": f"https://github.com/{repo_full_name}",
            },
        )

        sync, created = GithubRepositorySync.objects.get_or_create(
            repository=repo,
            project_id=project_id,
            workspace=workspace,
            defaults={
                "actor": request.user,
                "workspace_integration": workspace_integration,
                "credentials": {},
            },
        )

        if not created:
            return Response(
                {"error": "A sync for this project and repository already exists"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        return Response(
            {
                "id": str(sync.id),
                "project_id": str(sync.project_id),
                "repo_id": str(repo.repository_id),
                "repo_full_name": f"{repo.owner}/{repo.name}",
            },
            status=status.HTTP_201_CREATED,
        )

    @allow_permission([ROLE.ADMIN], level="WORKSPACE")
    def destroy(self, request, slug, pk):
        """Delete a GithubRepositorySync by its id."""
        sync = get_object_or_404(
            GithubRepositorySync,
            pk=pk,
            workspace__slug=slug,
        )
        sync.delete()
        return Response(status=status.HTTP_204_NO_CONTENT)
