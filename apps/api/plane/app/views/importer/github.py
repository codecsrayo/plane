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
from plane.utils.github_app import get_installation_access_token_result


class GithubRepositoriesEndpoint(BaseAPIView):
    """List GitHub repositories via GitHub App installation access token"""
    permission_classes = [WorkSpaceAdminPermission]

    def get(self, request, slug):
        # Resolve the installation token from the workspace's GitHub integration
        github_token = None
        github_token_error = None
        workspace_integration = None
        try:
            workspace = Workspace.objects.get(slug=slug)
            workspace_integration = WorkspaceIntegration.objects.filter(
                workspace=workspace,
                integration__provider="github",
            ).select_related("integration").first()
            if workspace_integration:
                installation_id = (workspace_integration.metadata or {}).get("installation_id")
                if installation_id:
                    github_token, github_token_error = get_installation_access_token_result(str(installation_id))
        except Exception:
            pass

        # Fall back to a PAT if no installation token is available
        if not github_token:
            github_token = request.query_params.get("token") or os.environ.get("GITHUB_ACCESS_TOKEN")

        if not github_token:
            if workspace_integration and (workspace_integration.metadata or {}).get("installation_id"):
                error_map = {
                    "github_app_not_configured": "GitHub App is not configured. Set GITHUB_APP_ID and GITHUB_APP_PRIVATE_KEY in God Mode integrations.",
                    "invalid_github_app_private_key": "GitHub App private key is invalid. Verify the base64-encoded PEM in God Mode integrations.",
                    "github_app_token_request_failed": "Failed to mint a GitHub installation token. Verify the app credentials and installation.",
                }
                if github_token_error in error_map:
                    return Response(
                        {"error": error_map[github_token_error]},
                        status=status.HTTP_400_BAD_REQUEST
                    )
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

        # Choose the correct endpoint based on token type.
        # Installation access tokens (from GitHub App) must use /installation/repositories.
        # Personal OAuth tokens use /user/repos.
        is_installation_token = bool(
            workspace_integration and (workspace_integration.metadata or {}).get("installation_id")
        )

        if is_installation_token:
            api_url = "https://api.github.com/installation/repositories"
            params: dict = {"per_page": per_page, "page": page}
        else:
            api_url = "https://api.github.com/user/repos"
            params = {"page": page, "per_page": per_page, "sort": "updated", "type": "all"}

        response = requests.get(api_url, headers=headers, params=params, timeout=10)

        if response.status_code != 200:
            return Response(
                {"error": "Failed to fetch repositories from GitHub"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        payload = response.json()
        # /installation/repositories returns { total_count, repositories: [...] }
        # /user/repos returns a plain array
        if isinstance(payload, dict):
            repos = payload.get("repositories", [])
            total_count = payload.get("total_count", len(repos))
        else:
            repos = payload
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
