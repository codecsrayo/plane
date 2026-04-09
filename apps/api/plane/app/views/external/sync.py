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

        PR → Plane state flow:
          1. Derive the logical gh_pr_state from the action + PR fields.
          2. Find all Plane issues linked to this PR via "Closes/Fixes/Resolves #N" in the body.
          3. For each linked issue, look up the GithubPRStateMapping and apply the target state,
             respecting the prevent_regression flag.
        """
        import re

        action = payload.get("action")
        pr = payload.get("pull_request", {})
        repo_id = payload.get("repository", {}).get("id")

        # ------------------------------------------------------------------
        # 1. Derive logical GitHub PR state
        # ------------------------------------------------------------------
        draft = pr.get("draft", False)

        if action in ("opened", "reopened"):
            gh_pr_state = "draft_open" if draft else "open"
        elif action == "converted_to_draft":
            gh_pr_state = "draft_open"
        elif action == "ready_for_review":
            gh_pr_state = "open"
        elif action == "review_requested":
            gh_pr_state = "review_requested"
        elif action == "closed":
            gh_pr_state = "merged" if pr.get("merged") else "closed"
        elif action == "synchronize":
            # Check review state to decide if ready_for_merge applies
            # We rely on the requested_reviewers / review decision field
            review_decision = pr.get("auto_merge") or pr.get("mergeable_state")
            if review_decision == "clean":
                gh_pr_state = "ready_for_merge"
            else:
                # Generic update — no state change needed
                return Response({"status": "ignored"}, status=status.HTTP_200_OK)
        else:
            return Response({"status": "ignored"}, status=status.HTTP_200_OK)

        # ------------------------------------------------------------------
        # 2. Find the repository sync record
        # ------------------------------------------------------------------
        sync = GithubRepositorySync.objects.filter(
            repository__repository_id=repo_id
        ).select_related("workspace_integration").first()

        if not sync:
            logger.debug("handle_pull_request: no sync for repo_id=%s", repo_id)
            return Response({"status": "ignored"}, status=status.HTTP_200_OK)

        # ------------------------------------------------------------------
        # 3. Extract linked Plane issue numbers from PR body
        #    Matches: closes #N, fixes #N, resolves #N (case-insensitive)
        # ------------------------------------------------------------------
        pr_body = pr.get("body") or ""
        linked_issue_numbers = list(
            set(
                int(n)
                for n in re.findall(
                    r"(?:closes?|fixes?|resolves?)\s+#(\d+)",
                    pr_body,
                    re.IGNORECASE,
                )
            )
        )

        if not linked_issue_numbers:
            logger.debug(
                "handle_pull_request: no linked issues in PR body for repo_id=%s pr_state=%s",
                repo_id,
                gh_pr_state,
            )
            return Response({"status": "ignored"}, status=status.HTTP_200_OK)

        # ------------------------------------------------------------------
        # 4. Look up mapping (one per project; all issues in the sync share the same project)
        # ------------------------------------------------------------------
        mapping = GithubPRStateMapping.objects.filter(
            workspace_integration=sync.workspace_integration,
            project_id=sync.project_id,
            github_pr_state=gh_pr_state,
        ).select_related("state").first()

        if not mapping:
            logger.debug(
                "handle_pull_request: no mapping for gh_pr_state=%s project=%s",
                gh_pr_state,
                sync.project_id,
            )
            return Response({"status": "ignored"}, status=status.HTTP_200_OK)

        target_state = mapping.state

        # ------------------------------------------------------------------
        # 5. Apply state to each linked issue, respecting prevent_regression
        # ------------------------------------------------------------------
        updated_count = 0
        for gh_issue_number in linked_issue_numbers:
            issue_sync = GithubIssueSync.objects.filter(
                github_issue_id=gh_issue_number,
                repository_sync=sync,
            ).select_related("issue__state").first()

            if not issue_sync:
                logger.debug(
                    "handle_pull_request: no issue sync for gh_issue_number=%s", gh_issue_number
                )
                continue

            issue = issue_sync.issue

            # Prevent regression: skip if target state is earlier in the workflow
            if mapping.prevent_regression and issue.state:
                if target_state.sequence < issue.state.sequence:
                    logger.debug(
                        "handle_pull_request: skipping regression for issue=%s "
                        "(current_seq=%s target_seq=%s)",
                        issue.id,
                        issue.state.sequence,
                        target_state.sequence,
                    )
                    continue

            # Skip if already in the target state
            if issue.state_id == target_state.id:
                continue

            issue.state_id = target_state.id
            issue.save(update_fields=["state_id", "updated_at"])
            updated_count += 1

        return Response(
            {"status": "success", "updated_issues": updated_count},
            status=status.HTTP_200_OK,
        )



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
