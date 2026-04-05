# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

from django.urls import path
from plane.app.views.importer import (
    GithubImporterEndpoint,
    GithubRepositoriesEndpoint,
    GitlabImporterEndpoint,
    GitlabRepositoriesEndpoint,
)

urlpatterns = [
    # List available GitHub repositories
    path(
        "workspaces/<str:slug>/importers/github/repositories/",
        GithubRepositoriesEndpoint.as_view(),
        name="github-repositories",
    ),
    # GitHub imports CRUD
    path(
        "workspaces/<str:slug>/importers/github/",
        GithubImporterEndpoint.as_view(),
        name="github-importer",
    ),
    path(
        "workspaces/<str:slug>/importers/github/<uuid:importer_id>/",
        GithubImporterEndpoint.as_view(),
        name="github-importer-detail",
    ),
    # List available GitLab repositories
    path(
        "workspaces/<str:slug>/importers/gitlab/repositories/",
        GitlabRepositoriesEndpoint.as_view(),
        name="gitlab-repositories",
    ),
    # GitLab imports CRUD
    path(
        "workspaces/<str:slug>/importers/gitlab/",
        GitlabImporterEndpoint.as_view(),
        name="gitlab-importer",
    ),
    path(
        "workspaces/<str:slug>/importers/gitlab/<uuid:importer_id>/",
        GitlabImporterEndpoint.as_view(),
        name="gitlab-importer-detail",
    ),
]
