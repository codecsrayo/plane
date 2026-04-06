# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

# Django imports
from django.db import models

# Module imports
from plane.db.models.base import BaseModel


class GithubPRStateMapping(BaseModel):
    workspace_integration = models.ForeignKey(
        "db.WorkspaceIntegration",
        on_delete=models.CASCADE,
        related_name="pr_state_mappings",
    )
    project = models.ForeignKey(
        "db.Project",
        on_delete=models.CASCADE,
        related_name="github_pr_state_mappings",
    )
    state = models.ForeignKey(
        "db.State",
        on_delete=models.CASCADE,
        related_name="github_pr_state_mappings",
    )
    github_pr_state = models.CharField(
        max_length=20,
        choices=[("open", "Open"), ("merged", "Merged"), ("closed", "Closed")],
    )

    class Meta:
        unique_together = [("workspace_integration", "project", "github_pr_state")]
        verbose_name = "GitHub PR State Mapping"
        verbose_name_plural = "GitHub PR State Mappings"
        db_table = "db_githubprstatemapping"
        ordering = ("-created_at",)
