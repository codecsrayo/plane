# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

from django.urls import path

from plane.app.views import (
    ProjectTemplateViewSet,
    ProjectSaveAsTemplateEndpoint,
    ProjectTemplateInstantiateEndpoint,
)

urlpatterns = [
    path(
        "workspaces/<str:slug>/project-templates/",
        ProjectTemplateViewSet.as_view({"get": "list", "post": "create"}),
        name="project-templates",
    ),
    path(
        "workspaces/<str:slug>/project-templates/<uuid:pk>/",
        ProjectTemplateViewSet.as_view(
            {
                "get": "retrieve",
                "patch": "partial_update",
                "delete": "destroy",
            }
        ),
        name="project-template-detail",
    ),
    path(
        "workspaces/<str:slug>/project-templates/<uuid:template_id>/instantiate/",
        ProjectTemplateInstantiateEndpoint.as_view(),
        name="project-template-instantiate",
    ),
    path(
        "workspaces/<str:slug>/projects/<uuid:project_id>/save-as-template/",
        ProjectSaveAsTemplateEndpoint.as_view(),
        name="project-save-as-template",
    ),
]
