# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

from django.db.models.signals import post_save
from django.dispatch import receiver
from plane.db.models import Issue, IssueComment, GithubIssueSync, GitlabIssueSync
from plane.bgtasks.sync_task import (
    sync_issue_to_github_task,
    sync_issue_to_gitlab_task,
    sync_comment_to_github_task,
    sync_comment_to_gitlab_task
)


@receiver(post_save, sender=Issue)
def sync_issue_to_external(sender, instance, created, **kwargs):
    if created:
        return

    # Trigger GitHub sync task
    if GithubIssueSync.objects.filter(issue=instance).exists():
        sync_issue_to_github_task.delay(str(instance.id))

    # Trigger GitLab sync task
    if GitlabIssueSync.objects.filter(issue=instance).exists():
        sync_issue_to_gitlab_task.delay(str(instance.id))


@receiver(post_save, sender=IssueComment)
def sync_comment_to_external(sender, instance, created, **kwargs):
    # Determine if this comment should be synced (e.g. not created by the sync itself)
    # We can check if it's already linked in CommentSync to decide if it's an update
    # or a new comment from Plane.

    # Check for GitHub sync
    github_issue_sync = GithubIssueSync.objects.filter(issue=instance.issue).first()
    if github_issue_sync:
        sync_comment_to_github_task.delay(str(instance.id))

    # Check for GitLab sync
    gitlab_issue_sync = GitlabIssueSync.objects.filter(issue=instance.issue).first()
    if gitlab_issue_sync:
        sync_comment_to_gitlab_task.delay(str(instance.id))
