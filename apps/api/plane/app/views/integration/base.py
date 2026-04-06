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
from plane.db.models import Integration, WorkspaceIntegration, Workspace, APIToken
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
