# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

import hashlib
import hmac
import logging
import os

from django.views.decorators.csrf import csrf_exempt
from django.utils.decorators import method_decorator
from rest_framework import status
from rest_framework.response import Response

from plane.app.views import BaseAPIView
from plane.db.models import (
    Issue,
    IssueComment,
    GithubRepositorySync,
    GithubIssueSync,
    GithubCommentSync,
    GithubPRStateMapping,
    GitlabRepositorySync,
    GitlabIssueSync,
    GitlabCommentSync,
)

logger = logging.getLogger(__name__)


def _get_webhook_secret(key: str) -> str:
    """Read a webhook secret from InstanceConfiguration DB, falling back to env var."""
    try:
        from plane.db.models import InstanceConfiguration
        config = InstanceConfiguration.objects.filter(key=key).first()
        if config and config.value:
            return config.value
    except Exception:
        pass
    return os.environ.get(key, "")


class GitHubWebhookEndpoint(BaseAPIView):
    authentication_classes = []
    permission_classes = []

    @method_decorator(csrf_exempt)
    def dispatch(self, *args, **kwargs):
        return super().dispatch(*args, **kwargs)

    def post(self, request, *args, **kwargs):
        signature = request.META.get("HTTP_X_HUB_SIGNATURE_256")
        if not signature:
            return Response({"error": "Signature missing"}, status=status.HTTP_400_BAD_REQUEST)

        # Retrieve webhook secret from InstanceConfiguration DB or environment
        webhook_secret = _get_webhook_secret("GITHUB_WEBHOOK_SECRET")
        if webhook_secret:
            # BUG FIX: was missing .hexdigest() — hmac.new() returns an HMAC object,
            # not a string; must call .hexdigest() to get the hex digest string.
            mac = hmac.new(
                webhook_secret.encode(),
                request.body,
                hashlib.sha256,
            )
            expected_signature = "sha256=" + mac.hexdigest()
            if not hmac.compare_digest(signature, expected_signature):
                return Response({"error": "Invalid signature"}, status=status.HTTP_403_FORBIDDEN)

        event = request.META.get("HTTP_X_GITHUB_EVENT")
        payload = request.data

        try:
            if event == "issues":
                return self.handle_issue(payload)
            elif event == "issue_comment":
                return self.handle_comment(payload)
            elif event == "pull_request":
                return self.handle_pull_request(payload)
        except Exception as exc:
            # Never crash with 500 — log and return 200 so GitHub doesn't retry endlessly
            logger.exception("Unhandled error processing GitHub webhook event=%s: %s", event, exc)

        return Response({"status": "ignored"}, status=status.HTTP_200_OK)

    def handle_issue(self, payload):
        action = payload.get("action")
        gh_issue = payload.get("issue")
        repo_id = payload.get("repository", {}).get("id")

        sync = GithubRepositorySync.objects.filter(repository__repository_id=repo_id).first()
        if not sync:
            return Response({"error": "Sync not configured"}, status=status.HTTP_404_NOT_FOUND)

        issue_sync = GithubIssueSync.objects.filter(github_issue_id=gh_issue["id"]).first()
        if issue_sync:
            if action in ["opened", "edited", "reopened", "closed"]:
                issue = issue_sync.issue
                issue.name = gh_issue["title"]
                issue.description_html = gh_issue.get("body") or ""
                issue.save()

        return Response({"status": "success"}, status=status.HTTP_200_OK)

    def handle_comment(self, payload):
        action = payload.get("action")
        gh_comment = payload.get("comment")
        gh_issue = payload.get("issue")

        issue_sync = GithubIssueSync.objects.filter(github_issue_id=gh_issue["id"]).first()
        if not issue_sync:
            return Response({"error": "Issue not synced"}, status=status.HTTP_404_NOT_FOUND)

        comment_sync = GithubCommentSync.objects.filter(repo_comment_id=gh_comment["id"]).first()
        if comment_sync:
            if action in ["edited", "deleted"]:
                if action == "deleted":
                    comment_sync.comment.delete()
                else:
                    comment = comment_sync.comment
                    comment.comment_html = gh_comment["body"]
                    comment.save()
        elif action == "created":
            new_comment = IssueComment.objects.create(
                issue=issue_sync.issue,
                project=issue_sync.project,
                workspace=issue_sync.workspace,
                comment_html=gh_comment["body"],
                actor=issue_sync.repository_sync.actor,
            )
            GithubCommentSync.objects.create(
                comment=new_comment,
                issue_sync=issue_sync,
                repo_comment_id=gh_comment["id"],
                project=issue_sync.project,
                workspace=issue_sync.workspace,
            )

        return Response({"status": "success"}, status=status.HTTP_200_OK)

    def handle_pull_request(self, payload):
        """
        Handle GitHub `pull_request` webhook events and update the linked Plane issue
        state according to the configured GithubPRStateMapping.
        """
        action = payload.get("action")
        pr = payload.get("pull_request", {})
        repo_id = payload.get("repository", {}).get("id")
        pr_number = pr.get("number")

        # Determine the logical PR state from the event action / payload fields
        if action in ("opened", "reopened"):
            gh_pr_state = "open"
        elif pr.get("merged") is True or action == "merged":
            gh_pr_state = "merged"
        elif action == "closed":
            gh_pr_state = "closed"
        else:
            # Ignore other actions (synchronize, labeled, assigned, …)
            return Response({"status": "ignored"}, status=status.HTTP_200_OK)

        # Find the repository sync record
        sync = GithubRepositorySync.objects.filter(repository__repository_id=repo_id).first()
        if not sync:
            logger.debug("handle_pull_request: no sync for repo_id=%s", repo_id)
            return Response({"status": "ignored"}, status=status.HTTP_200_OK)

        # Look up the PR state mapping for this workspace integration + project
        mapping = GithubPRStateMapping.objects.filter(
            workspace_integration=sync.workspace_integration,
            project_id=sync.project_id,
            github_pr_state=gh_pr_state,
        ).first()

        if not mapping:
            logger.debug(
                "handle_pull_request: no PR state mapping for gh_pr_state=%s project=%s",
                gh_pr_state,
                sync.project_id,
            )
            return Response({"status": "ignored"}, status=status.HTTP_200_OK)

        # Find the Plane issue linked to this PR number via GithubIssueSync
        issue_sync = GithubIssueSync.objects.filter(
            github_issue_id=pr_number,
            repository_sync=sync,
        ).first()

        if not issue_sync:
            logger.debug(
                "handle_pull_request: no issue sync for PR number=%s repo_id=%s",
                pr_number,
                repo_id,
            )
            return Response({"status": "ignored"}, status=status.HTTP_200_OK)

        # Update the issue state
        issue = issue_sync.issue
        issue.state_id = mapping.state_id
        issue.save(update_fields=["state_id", "updated_at"])

        return Response({"status": "success"}, status=status.HTTP_200_OK)


class GitLabWebhookEndpoint(BaseAPIView):
    authentication_classes = []
    permission_classes = []

    @method_decorator(csrf_exempt)
    def dispatch(self, *args, **kwargs):
        return super().dispatch(*args, **kwargs)

    def post(self, request, *args, **kwargs):
        token = request.META.get("HTTP_X_GITLAB_TOKEN")

        # Verify token
        gitlab_webhook_token = os.environ.get("GITLAB_WEBHOOK_TOKEN", "")
        if gitlab_webhook_token and token != gitlab_webhook_token:
            return Response({"error": "Invalid token"}, status=status.HTTP_403_FORBIDDEN)

        payload = request.data
        object_kind = payload.get("object_kind")

        try:
            if object_kind == "issue":
                return self.handle_issue(payload)
            elif object_kind == "note":
                return self.handle_note(payload)
            elif object_kind == "merge_request":
                return self.handle_merge_request(payload)
        except Exception as exc:
            # Never crash with 500 — log and return 200 so GitLab doesn't retry endlessly
            logger.exception(
                "Unhandled error processing GitLab webhook object_kind=%s: %s", object_kind, exc
            )

        return Response({"status": "ignored"}, status=status.HTTP_200_OK)

    def handle_issue(self, payload):
        attr = payload.get("object_attributes", {})
        gl_issue_id = attr.get("id")
        project_id = payload.get("project", {}).get("id")

        sync = GitlabRepositorySync.objects.filter(repository__repository_id=project_id).first()
        if not sync:
            return Response({"error": "Sync not configured"}, status=status.HTTP_404_NOT_FOUND)

        issue_sync = GitlabIssueSync.objects.filter(gitlab_issue_id=gl_issue_id).first()
        if issue_sync:
            issue = issue_sync.issue
            issue.name = attr.get("title")
            issue.description_html = attr.get("description") or ""
            issue.save()

        return Response({"status": "success"}, status=status.HTTP_200_OK)

    def handle_note(self, payload):
        attr = payload.get("object_attributes", {})
        if attr.get("noteable_type") != "Issue":
            return Response({"status": "ignored"}, status=status.HTTP_200_OK)

        gl_comment_id = attr.get("id")
        gl_issue_id = payload.get("issue", {}).get("id")

        issue_sync = GitlabIssueSync.objects.filter(gitlab_issue_id=gl_issue_id).first()
        if not issue_sync:
            return Response({"error": "Issue not synced"}, status=status.HTTP_404_NOT_FOUND)

        comment_sync = GitlabCommentSync.objects.filter(repo_comment_id=gl_comment_id).first()
        if comment_sync:
            comment = comment_sync.comment
            comment.comment_html = attr.get("note")
            comment.save()
        else:
            new_comment = IssueComment.objects.create(
                issue=issue_sync.issue,
                project=issue_sync.project,
                workspace=issue_sync.workspace,
                comment_html=attr.get("note"),
                actor=issue_sync.repository_sync.actor,
            )
            GitlabCommentSync.objects.create(
                comment=new_comment,
                issue_sync=issue_sync,
                repo_comment_id=gl_comment_id,
                project=issue_sync.project,
                workspace=issue_sync.workspace,
            )

        return Response({"status": "success"}, status=status.HTTP_200_OK)

    def handle_merge_request(self, payload):
        """
        Handle GitLab `merge_request` webhook events.

        TODO: Implement GitLab MR → Plane issue state mapping using a GitLab-specific
        PR state mapping model (or extend GithubPRStateMapping to be provider-agnostic).
        For now we log the event and skip state updates.

        GitLab MR states: object_attributes.state ∈ {opened, merged, closed, locked}
        """
        attr = payload.get("object_attributes", {})
        gl_mr_state = attr.get("state")
        project_id = payload.get("project", {}).get("id")
        mr_iid = attr.get("iid")  # internal MR number within the project

        logger.debug(
            "handle_merge_request: GitLab MR state=%s project_id=%s iid=%s — "
            "GitLab PR state mapping not yet implemented",
            gl_mr_state,
            project_id,
            mr_iid,
        )

        return Response({"status": "ignored"}, status=status.HTTP_200_OK)
