# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

# Django imports
from django.conf import settings
from django.db import models

# Module imports
from plane.db.models.base import BaseModel


class UserGithubConnection(BaseModel):
    """Stores the personal GitHub OAuth connection for a Plane user.

    This is separate from the workspace-level GitHub App installation
    (WorkspaceIntegration).  One user can connect their personal GitHub
    account independently of any workspace integration.
    """

    user = models.OneToOneField(
        settings.AUTH_USER_MODEL,
        on_delete=models.CASCADE,
        related_name="github_connection",
    )
    github_user_id = models.CharField(max_length=100)
    github_username = models.CharField(max_length=150)
    github_avatar_url = models.URLField(blank=True, default="")
    access_token = models.TextField()  # encrypted in production ideally

    class Meta:
        verbose_name = "User GitHub Connection"
        verbose_name_plural = "User GitHub Connections"
        db_table = "user_github_connections"
        ordering = ("-created_at",)

    def __str__(self):
        return f"{self.user_id} → {self.github_username}"
