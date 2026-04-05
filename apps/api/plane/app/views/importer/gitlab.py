# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

import os
import requests
from rest_framework import status
from rest_framework.response import Response
from plane.app.views import BaseAPIView
from plane.app.permissions import WorkSpaceAdminPermission
from plane.db.models import (
    Workspace,
    APIToken,
    Importer,
    Integration,
    WorkspaceIntegration,
    GitlabRepository,
    GitlabRepositorySync
)
from plane.bgtasks.importer_task import gitlab_importer_task


class GitlabRepositoriesEndpoint(BaseAPIView):
    """List GitLab repositories using the user's PAT"""
    permission_classes = [WorkSpaceAdminPermission]

    def get(self, request, slug):
        gitlab_host = os.environ.get("GITLAB_HOST", "https://gitlab.com").rstrip("/")
        gitlab_token = request.query_params.get("token") or os.environ.get("GITLAB_ACCESS_TOKEN")

        if not gitlab_token:
            return Response(
                {"error": "GitLab token not provided"},
                status=status.HTTP_400_BAD_REQUEST
            )

        page = int(request.query_params.get("page", 1))
        per_page = 30

        headers = {
            "PRIVATE-TOKEN": gitlab_token,
        }

        # Call GitLab API
        response = requests.get(
            f"{gitlab_host}/api/v4/projects",
            headers=headers,
            params={
                "page": page,
                "per_page": per_page,
                "membership": True,
                "simple": True,
                "order_by": "updated_at",
            },
            timeout=10,
        )

        if response.status_code != 200:
            return Response(
                {"error": f"Failed to fetch repositories from GitLab at {gitlab_host}"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        repos = response.json()
        total_count = len(repos)

        return Response(
            {
                "repositories": [
                    {
                        "id": str(repo["id"]),
                        "full_name": repo["path_with_namespace"],
                        "name": repo["name"],
                        "owner": repo["namespace"]["name"],
                        "description": repo.get("description", ""),
                        "private": repo.get("visibility") != "public",
                        "url": repo["web_url"],
                        "issues_count": repo.get("open_issues_count", 0),
                    }
                    for repo in repos
                ],
                "total_count": total_count,
                "page": page,
            },
            status=status.HTTP_200_OK,
        )


class GitlabImporterEndpoint(BaseAPIView):
    """Create, list, and delete GitLab imports"""
    permission_classes = [WorkSpaceAdminPermission]

    def get(self, request, slug):
        """List all imports for the workspace"""
        workspace = Workspace.objects.get(slug=slug)
        importers = Importer.objects.filter(
            workspace=workspace,
            service="gitlab",
        ).order_by("-created_at")

        data = [
            {
                "id": str(imp.id),
                "status": imp.status,
                "metadata": imp.metadata,
                "created_at": imp.created_at,
            }
            for imp in importers
        ]
        return Response(data, status=status.HTTP_200_OK)

    def post(self, request, slug):
        """Start a GitLab import"""
        workspace = Workspace.objects.get(slug=slug)
        project_id = request.data.get("project_id")
        gitlab_project_id = request.data.get("gitlab_project_id")
        gitlab_token = request.data.get("gitlab_token") or os.environ.get("GITLAB_ACCESS_TOKEN")

        if not all([project_id, gitlab_project_id, gitlab_token]):
            return Response(
                {"error": "project_id, gitlab_project_id, and gitlab_token are required"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        # Create a Plane API token for the importer
        api_token, _ = APIToken.objects.get_or_create(
            user=request.user,
            workspace=workspace,
            defaults={"label": "GitLab Importer Token"},
        )

        importer = Importer.objects.create(
            service="gitlab",
            project_id=project_id,
            workspace=workspace,
            initiated_by=request.user,
            token=api_token,
            metadata={
                "gitlab_project_id": gitlab_project_id,
            },
            config={
                "gitlab_token": gitlab_token,
                "gitlab_host": os.environ.get("GITLAB_HOST", "https://gitlab.com").rstrip("/"),
                "import_labels": request.data.get("import_labels", True),
                "import_comments": request.data.get("import_comments", True),
            },
        )

        # Ensure a GitlabRepositorySync exists for the project
        integration, _ = Integration.objects.get_or_create(
            provider="gitlab",
            defaults={"title": "GitLab"}
        )
        workspace_integration, _ = WorkspaceIntegration.objects.get_or_create(
            workspace=workspace,
            integration=integration,
            defaults={
                "actor": request.user,
                "api_token": api_token,
            }
        )

        repo, _ = GitlabRepository.objects.get_or_create(
            project_id=project_id,
            workspace=workspace,
            repository_id=gitlab_project_id,
            defaults={
                "name": f"GitLab Project {gitlab_project_id}",
                "owner": "gitlab",
            }
        )

        GitlabRepositorySync.objects.get_or_create(
            project_id=project_id,
            workspace=workspace,
            repository=repo,
            defaults={
                "actor": request.user,
                "workspace_integration": workspace_integration,
                "credentials": {
                    "token": gitlab_token,
                    "host": os.environ.get("GITLAB_HOST", "https://gitlab.com"),
                },
            }
        )

        # Enqueue the import task
        gitlab_importer_task.delay(str(importer.id))

        return Response(
            {"id": str(importer.id), "status": importer.status},
            status=status.HTTP_201_CREATED,
        )

    def delete(self, request, slug, importer_id):
        """Cancel/delete an import"""
        importer = Importer.objects.get(id=importer_id, workspace__slug=slug)
        importer.delete()
        return Response(status=status.HTTP_204_NO_CONTENT)
