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
    GithubRepository,
    GithubRepositorySync
)
from plane.bgtasks.importer_task import github_importer_task


class GithubRepositoriesEndpoint(BaseAPIView):
    """List GitHub repositories using the user's PAT"""
    permission_classes = [WorkSpaceAdminPermission]

    def get(self, request, slug):
        # Get the GitHub token from the request or configuration
        github_token = request.query_params.get("token") or os.environ.get("GITHUB_ACCESS_TOKEN")
        if not github_token:
            return Response(
                {"error": "GitHub token not provided"},
                status=status.HTTP_400_BAD_REQUEST
            )

        page = int(request.query_params.get("page", 1))
        per_page = 30

        headers = {
            "Authorization": f"Bearer {github_token}",
            "Accept": "application/vnd.github+json",
        }

        # Call GitHub API
        response = requests.get(
            "https://api.github.com/user/repos",
            headers=headers,
            params={
                "page": page,
                "per_page": per_page,
                "sort": "updated",
                "type": "all",
            },
            timeout=10,
        )

        if response.status_code != 200:
            return Response(
                {"error": "Failed to fetch repositories from GitHub"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        repos = response.json()
        # Simplified total count from the current page length
        total_count = len(repos)

        return Response(
            {
                "repositories": [
                    {
                        "id": str(repo["id"]),
                        "full_name": repo["full_name"],
                        "name": repo["name"],
                        "owner": repo["owner"]["login"],
                        "description": repo.get("description", ""),
                        "private": repo["private"],
                        "url": repo["html_url"],
                        "issues_count": repo.get("open_issues_count", 0),
                    }
                    for repo in repos
                ],
                "total_count": total_count,
                "page": page,
            },
            status=status.HTTP_200_OK,
        )


class GithubImporterEndpoint(BaseAPIView):
    """Create, list, and delete GitHub imports"""
    permission_classes = [WorkSpaceAdminPermission]

    def get(self, request, slug):
        """List all imports for the workspace"""
        workspace = Workspace.objects.get(slug=slug)
        importers = Importer.objects.filter(
            workspace=workspace,
            service="github",
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
        """Start a GitHub import"""
        workspace = Workspace.objects.get(slug=slug)
        project_id = request.data.get("project_id")
        repo_owner = request.data.get("owner")
        repo_name = request.data.get("repo")
        github_token = request.data.get("github_token") or os.environ.get("GITHUB_ACCESS_TOKEN")

        if not all([project_id, repo_owner, repo_name, github_token]):
            return Response(
                {"error": "project_id, owner, repo, and github_token are required"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        # Create a Plane API token for the importer
        api_token, _ = APIToken.objects.get_or_create(
            user=request.user,
            workspace=workspace,
            defaults={"label": "GitHub Importer Token"},
        )

        importer = Importer.objects.create(
            service="github",
            project_id=project_id,
            workspace=workspace,
            initiated_by=request.user,
            token=api_token,
            metadata={
                "owner": repo_owner,
                "name": repo_name,
                "repository": f"{repo_owner}/{repo_name}",
            },
            config={
                "github_token": github_token,
                "import_labels": request.data.get("import_labels", True),
                "import_comments": request.data.get("import_comments", True),
            },
        )

        # Ensure a GithubRepositorySync exists for the project to enable bidirectional sync
        integration = Integration.objects.filter(provider="github").first()
        if integration:
            workspace_integration, _ = WorkspaceIntegration.objects.get_or_create(
                workspace=workspace,
                integration=integration,
                defaults={
                    "actor": request.user,
                    "api_token": api_token,
                }
            )

            # Use GitHub project ID from some source if available, or just use metadata
            # For simplicity, we ensure a GithubRepository and its Sync record exist
            repo, _ = GithubRepository.objects.get_or_create(
                project_id=project_id,
                workspace=workspace,
                name=repo_name,
                owner=repo_owner,
                defaults={
                    "repository_id": 0, # Should ideally be fetched from GitHub
                    "url": f"https://github.com/{repo_owner}/{repo_name}",
                }
            )

            GithubRepositorySync.objects.get_or_create(
                project_id=project_id,
                workspace=workspace,
                repository=repo,
                defaults={
                    "actor": request.user,
                    "workspace_integration": workspace_integration,
                    "credentials": {"token": github_token},
                }
            )

        # Enqueue the import task
        github_importer_task.delay(str(importer.id))

        return Response(
            {"id": str(importer.id), "status": importer.status},
            status=status.HTTP_201_CREATED,
        )

    def delete(self, request, slug, importer_id):
        """Cancel/delete an import"""
        importer = Importer.objects.get(id=importer_id, workspace__slug=slug)
        importer.delete()
        return Response(status=status.HTTP_204_NO_CONTENT)
