# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

# Python imports
import os
import requests

# Third party imports
from django.db import IntegrityError, transaction
from django.http import HttpResponse
from django.shortcuts import get_object_or_404, redirect
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
    GithubPRStateMapping,
    UserGithubConnection,
    WorkspaceMember,
)
from plane.app.serializers import (
    IntegrationSerializer,
    WorkspaceIntegrationSerializer,
    GithubPRStateMappingSerializer,
)
from plane.app.permissions import WorkSpaceAdminPermission, ROLE, allow_permission
from plane.license.utils.instance_value import get_configuration_value
from plane.authentication.utils.host import base_host
from plane.utils.exception_logger import log_exception


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
        try:
            with transaction.atomic():
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
        except IntegrityError:
            # Race condition on simultaneous installs — fetch the existing record.
            workspace_integration = WorkspaceIntegration.objects.get(
                workspace=workspace,
                integration=integration,
            )
            created = False

        if not created:
            # Update metadata/config with fresh OAuth data in case of re-install
            workspace_integration.metadata = metadata
            workspace_integration.config = config
            workspace_integration.save(update_fields=["metadata", "config"])

        serializer = WorkspaceIntegrationSerializer(workspace_integration)
        return Response(serializer.data, status=status.HTTP_201_CREATED if created else status.HTTP_200_OK)


def _postmessage_html(success: bool, message_type: str, error: str = "") -> HttpResponse:
    """
    Return a minimal HTML page that sends a postMessage to the opener window
    and closes itself. Used by OAuth/App callback endpoints so that the popup
    flow works correctly regardless of which URL GitHub is configured to call.
    """
    payload = f'{{"type": "{message_type}", "success": {"true" if success else "false"}'
    if error:
        safe_error = error.replace('"', '\\"')
        payload += f', "error": "{safe_error}"'
    payload += "}"

    html = f"""<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>Connecting…</title></head>
<body>
<script>
(function () {{
  try {{
    window.opener && window.opener.postMessage({payload}, window.location.origin);
  }} catch (e) {{}}
  window.close();
}})();
</script>
<p style="font-family:sans-serif;text-align:center;margin-top:4rem;">
  {"Integration connected successfully. You may close this window." if success else "An error occurred. You may close this window."}
</p>
</body>
</html>"""
    return HttpResponse(html, content_type="text/html")


class GithubAppCallbackEndpoint(BaseAPIView):
    """
    Direct GitHub App installation callback.

    Configure the GitHub App "Setup URL" to point here:
      https://{host}/api/github/callback/

    GitHub redirects the popup window here after installation:
      GET /api/github/callback/?installation_id=XXX&setup_action=install&state={workspaceSlug}

    This view:
    1. Reads installation_id and state (workspaceSlug) from query params
    2. Creates/updates the WorkspaceIntegration record
    3. Returns an HTML page that sends a postMessage to the opener and closes the popup.
       (previously used redirect() which left the popup open and broke the parent flow)
    """

    authentication_classes = []  # GitHub redirects unauthenticated
    permission_classes = []

    def get(self, request):
        installation_id = request.GET.get("installation_id")
        setup_action = request.GET.get("setup_action", "install")
        workspace_slug = request.GET.get("state")

        if not installation_id or not workspace_slug:
            return _postmessage_html(
                success=False,
                message_type="github-integration",
                error="Missing installation_id or workspace context.",
            )

        try:
            workspace = Workspace.objects.get(slug=workspace_slug)
            integration = Integration.objects.get(provider="github")

            # This callback is unauthenticated — find the first workspace admin to act as actor.
            admin_member = (
                WorkspaceMember.objects.filter(workspace=workspace, role__gte=20)
                .select_related("member")
                .order_by("created_at")
                .first()
            )

            if not admin_member:
                # Cannot save WorkspaceIntegration without a valid actor (NOT NULL field).
                # This should never happen in a healthy workspace but we must not return
                # success=True and silently discard the installation_id.
                import logging
                logging.getLogger(__name__).error(
                    "GithubAppCallbackEndpoint: no admin member found for workspace %s — "
                    "installation_id %s cannot be persisted.",
                    workspace_slug,
                    installation_id,
                )
                return _postmessage_html(
                    success=False,
                    message_type="github-integration",
                    error="No workspace admin found. Please contact your workspace administrator.",
                )

            actor = admin_member.member

            api_token, _ = APIToken.objects.get_or_create(
                user=actor,
                workspace=workspace,
                defaults={"label": f"{integration.title} Integration Token"},
            )

            # Both actor and api_token are guaranteed non-None at this point.
            update_defaults = {
                "metadata": {"installation_id": installation_id, "setup_action": setup_action},
                "config": {"installation_id": installation_id},
                "actor": actor,
                "api_token": api_token,
            }

            try:
                with transaction.atomic():
                    WorkspaceIntegration.objects.update_or_create(
                        workspace=workspace,
                        integration=integration,
                        defaults=update_defaults,
                    )
            except IntegrityError:
                # Race condition — record just created by a concurrent request; update in place.
                WorkspaceIntegration.objects.filter(
                    workspace=workspace,
                    integration=integration,
                ).update(
                    metadata=update_defaults["metadata"],
                    config=update_defaults["config"],
                )

        except (Workspace.DoesNotExist, Integration.DoesNotExist) as e:
            import logging
            logging.getLogger(__name__).warning("GithubAppCallbackEndpoint: %s", e)
            return _postmessage_html(
                success=False,
                message_type="github-integration",
                error="Workspace or integration not found.",
            )
        except Exception as e:
            import logging
            logging.getLogger(__name__).error("GithubAppCallbackEndpoint: %s", e, exc_info=True)
            return _postmessage_html(
                success=False,
                message_type="github-integration",
                error="Installation failed. Please try again.",
            )

        return _postmessage_html(success=True, message_type="github-integration")


class UserGithubConnectionView(BaseAPIView):
    """
    Exchange a GitHub OAuth code for a personal user token and persist the
    authenticated user's GitHub connection.
    """

    def post(self, request):
        code = request.data.get("code")

        if not code:
            return Response(
                {"error": "code is required"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET = get_configuration_value(
            [
                {"key": "GITHUB_CLIENT_ID", "default": os.environ.get("GITHUB_CLIENT_ID", "")},
                {"key": "GITHUB_CLIENT_SECRET", "default": os.environ.get("GITHUB_CLIENT_SECRET", "")},
            ]
        )

        if not (GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET):
            return Response(
                {"error": "GitHub OAuth is not configured"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        redirect_uri = f"{base_host(request=request, is_app=True).rstrip('/')}/auth/github/user-callback/"

        try:
            token_response = requests.post(
                "https://github.com/login/oauth/access_token",
                data={
                    "client_id": GITHUB_CLIENT_ID,
                    "client_secret": GITHUB_CLIENT_SECRET,
                    "code": code,
                    "redirect_uri": redirect_uri,
                },
                headers={"Accept": "application/json"},
                timeout=10,
            )
            token_response.raise_for_status()
            token_data = token_response.json()
        except requests.RequestException as exc:
            log_exception(exc)
            return Response(
                {"error": "Failed to exchange GitHub authorization code"},
                status=status.HTTP_502_BAD_GATEWAY,
            )

        access_token = token_data.get("access_token")
        if not access_token:
            return Response(
                {"error": token_data.get("error_description") or "GitHub did not return an access token"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        try:
            user_response = requests.get(
                "https://api.github.com/user",
                headers={
                    "Authorization": f"Bearer {access_token}",
                    "Accept": "application/json",
                },
                timeout=10,
            )
            user_response.raise_for_status()
            github_user = user_response.json()
        except requests.RequestException as exc:
            log_exception(exc)
            return Response(
                {"error": "Failed to fetch GitHub user profile"},
                status=status.HTTP_502_BAD_GATEWAY,
            )

        github_user_id = github_user.get("id")
        github_username = github_user.get("login")

        if not github_user_id or not github_username:
            return Response(
                {"error": "GitHub returned an incomplete user profile"},
                status=status.HTTP_502_BAD_GATEWAY,
            )

        connection, created = UserGithubConnection.objects.update_or_create(
            user=request.user,
            defaults={
                "github_user_id": str(github_user_id),
                "github_username": github_username,
                "github_avatar_url": github_user.get("avatar_url", ""),
                "access_token": access_token,
            },
        )

        return Response(
            {
                "id": str(connection.id),
                "github_user_id": connection.github_user_id,
                "github_username": connection.github_username,
                "github_avatar_url": connection.github_avatar_url,
                "created": created,
            },
            status=status.HTTP_201_CREATED if created else status.HTTP_200_OK,
        )


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
        ).select_related("repository", "workspace_integration", "actor", "project")

    @allow_permission([ROLE.ADMIN, ROLE.MEMBER], level="WORKSPACE")
    def list(self, request, slug):
        """Return all repo syncs for this workspace's GitHub integration."""
        syncs = self.get_queryset()
        data = [
            {
                "id": str(s.id),
                "project_id": str(s.project_id),
                "project_name": s.project.name,
                "project_identifier": s.project.identifier,
                "repo_id": str(s.repository.repository_id),
                "repo_full_name": f"{s.repository.owner}/{s.repository.name}",
                "repo_name": s.repository.name,
                "repo_owner": s.repository.owner,
                "sync_direction": (s.credentials or {}).get("sync_direction", "bidirectional"),
                "issue_open_state": (s.credentials or {}).get("issue_open_state"),
                "issue_closed_state": (s.credentials or {}).get("issue_closed_state"),
                "created_at": s.created_at,
            }
            for s in syncs
        ]
        return Response(data, status=status.HTTP_200_OK)

    @allow_permission([ROLE.ADMIN], level="WORKSPACE")
    def create(self, request, slug):
        """
        Create a GithubRepositorySync linking a project with a GitHub repo.
        Body: {
            repo_id, repo_full_name, project_id,
            issue_open_state?,    # Plane state UUID for open GitHub issues
            issue_closed_state?,  # Plane state UUID for closed GitHub issues
            sync_direction?       # "bidirectional" | "unidirectional"
        }
        """
        repo_id_raw = request.data.get("repo_id")
        repo_full_name = request.data.get("repo_full_name", "")
        project_id = request.data.get("project_id")

        if not all([repo_id_raw, project_id]):
            return Response(
                {"error": "repo_id and project_id are required"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        # Validate repo_id is a numeric GitHub repository ID
        try:
            repo_id_int = int(repo_id_raw)
        except (TypeError, ValueError):
            return Response(
                {"error": "repo_id must be a numeric GitHub repository ID"},
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

        # Get or create the GithubRepository record.
        # Must use all_objects to include soft-deleted rows — otherwise get_or_create
        # ignores them and tries to INSERT a duplicate, which trips the DB unique constraint.
        repo = GithubRepository.all_objects.filter(
            repository_id=repo_id_int,
            project_id=project_id,
        ).first()

        if repo is None:
            repo = GithubRepository.objects.create(
                repository_id=repo_id_int,
                project_id=project_id,
                name=repo_name,
                owner=repo_owner,
                url=f"https://github.com/{repo_full_name}",
            )
        elif repo.deleted_at is not None:
            # Resurrect the soft-deleted repo record so it can be reused.
            repo.deleted_at = None
            repo.name = repo_name
            repo.owner = repo_owner
            repo.url = f"https://github.com/{repo_full_name}"
            repo.save(update_fields=["deleted_at", "name", "owner", "url"])

        # Build credentials dict — store optional sync config alongside any future tokens
        credentials = {
            "sync_direction": request.data.get("sync_direction", "bidirectional"),
        }
        if request.data.get("issue_open_state"):
            credentials["issue_open_state"] = request.data["issue_open_state"]
        if request.data.get("issue_closed_state"):
            credentials["issue_closed_state"] = request.data["issue_closed_state"]

        # Check for existing sync — including soft-deleted ones (all_objects bypasses the
        # soft-delete manager so we don't hit the OneToOneField DB unique constraint).
        existing_sync = GithubRepositorySync.all_objects.filter(
            repository=repo,
            project_id=project_id,
            workspace=workspace,
        ).first()

        if existing_sync is not None:
            if existing_sync.deleted_at is None:
                # Genuinely active sync already exists — reject the duplicate.
                return Response(
                    {"error": "A sync for this project and repository already exists"},
                    status=status.HTTP_400_BAD_REQUEST,
                )
            # Resurrect the soft-deleted sync with fresh settings.
            existing_sync.deleted_at = None
            existing_sync.actor = request.user
            existing_sync.workspace_integration = workspace_integration
            existing_sync.credentials = credentials
            existing_sync.save(
                update_fields=["deleted_at", "actor", "workspace_integration", "credentials"]
            )
            sync = existing_sync
        else:
            sync = GithubRepositorySync.objects.create(
                repository=repo,
                project_id=project_id,
                workspace=workspace,
                actor=request.user,
                workspace_integration=workspace_integration,
                credentials=credentials,
            )

        # Auto-register the Plane webhook on GitHub so we receive real-time events.
        # Failure here must not roll back the sync creation.
        self._register_github_webhook(
            workspace_integration=workspace_integration,
            owner=repo_owner,
            repo_name=repo_name,
        )

        return Response(
            {
                "id": str(sync.id),
                "project_id": str(sync.project_id),
                "project_name": sync.project.name,
                "project_identifier": sync.project.identifier,
                "repo_id": str(repo.repository_id),
                "repo_full_name": f"{repo.owner}/{repo.name}",
                "sync_direction": credentials.get("sync_direction", "bidirectional"),
                "issue_open_state": credentials.get("issue_open_state"),
                "issue_closed_state": credentials.get("issue_closed_state"),
            },
            status=status.HTTP_201_CREATED,
        )

    def _register_github_webhook(
        self, *, workspace_integration, owner: str, repo_name: str
    ) -> None:
        """
        Register a webhook on GitHub for the given repo so that Plane receives
        real-time issue / PR / comment events.

        All failures are swallowed so that sync creation is never blocked.
        """
        try:
            from plane.utils.github_app import get_installation_access_token

            installation_id = workspace_integration.metadata.get("installation_id")
            if not installation_id:
                return

            token = get_installation_access_token(installation_id)
            if not token:
                return

            webhook_secret = os.environ.get("GITHUB_WEBHOOK_SECRET", "")
            web_url = os.environ.get("WEB_URL", "")
            if not web_url:
                return

            requests.post(
                f"https://api.github.com/repos/{owner}/{repo_name}/hooks",
                headers={
                    "Authorization": f"Bearer {token}",
                    "Accept": "application/vnd.github+json",
                    "X-GitHub-Api-Version": "2022-11-28",
                },
                json={
                    "name": "web",
                    "config": {
                        "url": f"{web_url}/api/github-webhook/",
                        "content_type": "json",
                        "secret": webhook_secret,
                    },
                    "events": ["issues", "pull_request", "issue_comment"],
                    "active": True,
                },
                timeout=10,
            )
        except Exception:
            # Don't fail the sync creation if webhook registration fails
            pass

    @allow_permission([ROLE.ADMIN], level="WORKSPACE")
    def destroy(self, request, slug, pk):
        """Delete a GithubRepositorySync (and its orphaned GithubRepository) by id."""
        sync = get_object_or_404(
            GithubRepositorySync,
            pk=pk,
            workspace__slug=slug,
        )
        # Capture the repo before deleting the sync so we can clean it up too.
        # GithubRepository is a pure bridge record — without its sync it is orphaned.
        repo = sync.repository
        sync.delete()
        repo.delete()
        return Response(status=status.HTTP_204_NO_CONTENT)


class GithubPRStateMappingViewSet(BaseViewSet):
    """
    Manage PR state mappings for a WorkspaceIntegration.

    GET    /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/
    POST   /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/
    DELETE /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{id}/
    """

    serializer_class = GithubPRStateMappingSerializer

    def get_queryset(self):
        return GithubPRStateMapping.objects.filter(
            workspace_integration__workspace__slug=self.kwargs["slug"],
            workspace_integration_id=self.kwargs["wi_id"],
        ).select_related("state", "project")

    @allow_permission([ROLE.ADMIN, ROLE.MEMBER], level="WORKSPACE")
    def list(self, request, slug, wi_id):
        mappings = self.get_queryset()
        serializer = GithubPRStateMappingSerializer(mappings, many=True)
        return Response(serializer.data, status=status.HTTP_200_OK)

    @allow_permission([ROLE.ADMIN], level="WORKSPACE")
    def create(self, request, slug, wi_id):
        wi = get_object_or_404(
            WorkspaceIntegration,
            pk=wi_id,
            workspace__slug=slug,
        )
        serializer = GithubPRStateMappingSerializer(data=request.data)
        if not serializer.is_valid():
            return Response(serializer.errors, status=status.HTTP_400_BAD_REQUEST)
        serializer.save(workspace_integration=wi)
        return Response(serializer.data, status=status.HTTP_201_CREATED)

    @allow_permission([ROLE.ADMIN], level="WORKSPACE")
    def destroy(self, request, slug, wi_id, pk):
        mapping = get_object_or_404(
            GithubPRStateMapping,
            pk=pk,
            workspace_integration__pk=wi_id,
            workspace_integration__workspace__slug=slug,
        )
        mapping.delete()
        return Response(status=status.HTTP_204_NO_CONTENT)
