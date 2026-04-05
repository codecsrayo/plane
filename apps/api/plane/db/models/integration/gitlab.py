# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

# Python imports

# Django imports
from django.db import models

# Module imports
from plane.db.models.project import ProjectBaseModel


class GitlabRepository(ProjectBaseModel):
    name = models.CharField(max_length=500)
    url = models.URLField(null=True)
    config = models.JSONField(default=dict)
    repository_id = models.BigIntegerField()
    owner = models.CharField(max_length=500)

    def __str__(self):
        """Return the repo name"""
        return f"{self.name}"

    class Meta:
        verbose_name = "Gitlab Repository"
        verbose_name_plural = "Gitlab Repositories"
        db_table = "gitlab_repositories"
        ordering = ("-created_at",)


class GitlabRepositorySync(ProjectBaseModel):
    repository = models.OneToOneField("db.GitlabRepository", on_delete=models.CASCADE, related_name="syncs")
    credentials = models.JSONField(default=dict)
    # Bot user
    actor = models.ForeignKey("db.User", related_name="gitlab_user_syncs", on_delete=models.CASCADE)
    workspace_integration = models.ForeignKey(
        "db.WorkspaceIntegration", related_name="gitlab_syncs", on_delete=models.CASCADE
    )
    label = models.ForeignKey("db.Label", on_delete=models.SET_NULL, null=True, related_name="gitlab_repo_syncs")

    def __str__(self):
        """Return the repo sync"""
        return f"{self.repository.name} <{self.project.name}>"

    class Meta:
        unique_together = ["project", "repository"]
        verbose_name = "Gitlab Repository Sync"
        verbose_name_plural = "Gitlab Repository Syncs"
        db_table = "gitlab_repository_syncs"
        ordering = ("-created_at",)


class GitlabIssueSync(ProjectBaseModel):
    repo_issue_id = models.BigIntegerField()
    gitlab_issue_id = models.BigIntegerField()
    issue_url = models.URLField(blank=False)
    issue = models.ForeignKey("db.Issue", related_name="gitlab_syncs", on_delete=models.CASCADE)
    repository_sync = models.ForeignKey("db.GitlabRepositorySync", related_name="issue_syncs", on_delete=models.CASCADE)

    def __str__(self):
        """Return the gitlab issue sync"""
        return f"{self.repository_sync.repository.name}-{self.project.name}-{self.issue.name}"

    class Meta:
        unique_together = ["repository_sync", "issue"]
        verbose_name = "Gitlab Issue Sync"
        verbose_name_plural = "Gitlab Issue Syncs"
        db_table = "gitlab_issue_syncs"
        ordering = ("-created_at",)


class GitlabCommentSync(ProjectBaseModel):
    repo_comment_id = models.BigIntegerField()
    comment = models.ForeignKey("db.IssueComment", related_name="gitlab_comment_syncs", on_delete=models.CASCADE)
    issue_sync = models.ForeignKey("db.GitlabIssueSync", related_name="comment_syncs", on_delete=models.CASCADE)

    def __str__(self):
        """Return the gitlab issue sync"""
        return f"{self.comment.id}"

    class Meta:
        unique_together = ["issue_sync", "comment"]
        verbose_name = "Gitlab Comment Sync"
        verbose_name_plural = "Gitlab Comment Syncs"
        db_table = "gitlab_comment_syncs"
        ordering = ("-created_at",)
