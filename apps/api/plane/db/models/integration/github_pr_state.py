# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

# Django imports
from django.db import models

# Module imports
from plane.db.models.base import BaseModel


class GithubPRStateMapping(BaseModel):
    GITHUB_PR_STATE_CHOICES = [
        ("draft_open", "Draft Open"),
        ("open", "Open"),
        ("review_requested", "Review Requested"),
        ("ready_for_merge", "Ready for Merge"),
        ("merged", "Merged"),
        ("closed", "Closed"),
    ]

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
        choices=GITHUB_PR_STATE_CHOICES,
    )
    prevent_regression = models.BooleanField(
        default=False,
        help_text="Prevent issues from moving to an earlier state due to PR updates",
    )

    class Meta:
        unique_together = [("workspace_integration", "project", "github_pr_state")]
        verbose_name = "GitHub PR State Mapping"
        verbose_name_plural = "GitHub PR State Mappings"
        db_table = "db_githubprstatemapping"
        ordering = ("-created_at",)
