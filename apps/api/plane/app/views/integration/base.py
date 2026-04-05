# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

# Third party imports
from django.shortcuts import get_object_or_404
from rest_framework import status
from rest_framework.response import Response
from rest_framework.permissions import IsAuthenticated

# Module imports
from plane.app.views.base import BaseViewSet
from plane.db.models import Integration, WorkspaceIntegration, Workspace, APIToken
from plane.app.serializers import IntegrationSerializer, WorkspaceIntegrationSerializer
from plane.app.permissions import WorkSpaceAdminPermission, ROLE, allow_permission


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
