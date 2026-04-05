# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

import json
import hashlib
import hmac
import os
from django.conf import settings
from django.views.decorators.csrf import csrf_exempt
from django.utils.decorators import method_decorator
from rest_framework import status
from rest_framework.response import Response
from plane.app.views import BaseAPIView
from plane.db.models import (
    Issue, IssueComment, State, Label, IssueLabel,
    GithubRepositorySync, GithubIssueSync, GithubCommentSync,
    GitlabRepositorySync, GitlabIssueSync, GitlabCommentSync
)


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

        # Retrieve webhook secret from environment or DB
        webhook_secret = os.environ.get("GITHUB_WEBHOOK_SECRET", "")
        if webhook_secret:
            expected_signature = "sha256=" + hmac.new(
                webhook_secret.encode(),
                request.body,
                hashlib.sha256
            ).hexdigest()
            if not hmac.compare_digest(signature, expected_signature):
                return Response({"error": "Invalid signature"}, status=status.HTTP_403_FORBIDDEN)

        event = request.META.get("HTTP_X_GITHUB_EVENT")
        payload = request.data

        if event == "issues":
            return self.handle_issue(payload)
        elif event == "issue_comment":
            return self.handle_comment(payload)

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
                # Logic for status/labels could be added here
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
                workspace=issue_sync.workspace
            )

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

        if object_kind == "issue":
            return self.handle_issue(payload)
        elif object_kind == "note":
            return self.handle_note(payload)

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
                workspace=issue_sync.workspace
            )

        return Response({"status": "success"}, status=status.HTTP_200_OK)
