# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

from django.urls import path
from plane.app.views import (
    IntegrationViewSet,
    WorkspaceIntegrationViewSet,
    GithubRepoSyncViewSet,
)

urlpatterns = [
    # Global integrations
    path(
        "integrations/",
        IntegrationViewSet.as_view({"get": "list"}),
        name="integrations-list",
    ),
    # Workspace specific integrations
    path(
        "workspaces/<str:slug>/workspace-integrations/",
        WorkspaceIntegrationViewSet.as_view({"get": "list", "post": "create"}),
        name="workspace-integrations-list",
    ),
    path(
        "workspaces/<str:slug>/workspace-integrations/<uuid:pk>/",
        WorkspaceIntegrationViewSet.as_view({"get": "retrieve", "patch": "partial_update", "delete": "destroy"}),
        name="workspace-integrations-detail",
    ),
    # Some frontends might use provider slug instead of UUID
    path(
        "workspaces/<str:slug>/workspace-integrations/<str:provider>/provider/",
        WorkspaceIntegrationViewSet.as_view({"delete": "provider_destroy"}),
        name="workspace-integrations-provider-delete",
    ),
    # OAuth callback install: POST with installation_id / code after user completes OAuth popup
    path(
        "workspaces/<str:slug>/workspace-integrations/<str:provider>/install/",
        WorkspaceIntegrationViewSet.as_view({"post": "provider_install"}),
        name="workspace-integrations-provider-install",
    ),
    # GitHub repository sync management
    path(
        "workspaces/<str:slug>/workspace-integrations/github/repo-syncs/",
        GithubRepoSyncViewSet.as_view({"get": "list", "post": "create"}),
        name="github-repo-syncs-list",
    ),
    path(
        "workspaces/<str:slug>/workspace-integrations/github/repo-syncs/<uuid:pk>/",
        GithubRepoSyncViewSet.as_view({"delete": "destroy"}),
        name="github-repo-syncs-detail",
    ),
]
