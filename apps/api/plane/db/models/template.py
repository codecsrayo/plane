# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

from django.db import models

from .base import BaseModel


class ProjectTemplate(BaseModel):
    workspace = models.ForeignKey(
        "db.Workspace",
        on_delete=models.CASCADE,
        related_name="workspace_project_templates",
    )
    name = models.CharField(max_length=255)
    description = models.TextField(blank=True)
    # Snapshot of project structure at template creation time
    template_data = models.JSONField(default=dict)

    class Meta:
        verbose_name = "Project Template"
        verbose_name_plural = "Project Templates"
        db_table = "project_templates"
        ordering = ("-created_at",)

    def __str__(self):
        return self.name
