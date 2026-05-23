# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

# Django imports
from django.conf import settings
from django.db import models

# Module imports
from .base import BaseModel

TEAMSPACE_ROLE_CHOICES = ((20, "Admin"), (15, "Member"), (5, "Guest"))


class Teamspace(BaseModel):
    name = models.CharField(max_length=255, verbose_name="Teamspace Name")
    description = models.TextField(verbose_name="Teamspace Description", blank=True)
    workspace = models.ForeignKey(
        "db.Workspace",
        on_delete=models.CASCADE,
        related_name="workspace_teamspaces",
    )
    logo_props = models.JSONField(default=dict)

    class Meta:
        unique_together = ["name", "workspace", "deleted_at"]
        constraints = [
            models.UniqueConstraint(
                fields=["name", "workspace"],
                condition=models.Q(deleted_at__isnull=True),
                name="teamspace_unique_name_workspace_when_deleted_at_null",
            )
        ]
        verbose_name = "Teamspace"
        verbose_name_plural = "Teamspaces"
        db_table = "teamspaces"
        ordering = ("-created_at",)

    def __str__(self):
        return f"{self.name} <{self.workspace.name}>"


class TeamspaceMember(BaseModel):
    teamspace = models.ForeignKey(
        Teamspace,
        on_delete=models.CASCADE,
        related_name="teamspace_members",
    )
    member = models.ForeignKey(
        settings.AUTH_USER_MODEL,
        on_delete=models.CASCADE,
        related_name="member_teamspaces",
    )
    role = models.PositiveSmallIntegerField(choices=TEAMSPACE_ROLE_CHOICES, default=15)

    class Meta:
        unique_together = ["teamspace", "member", "deleted_at"]
        constraints = [
            models.UniqueConstraint(
                fields=["teamspace", "member"],
                condition=models.Q(deleted_at__isnull=True),
                name="teamspace_member_unique_teamspace_member_when_deleted_at_null",
            )
        ]
        verbose_name = "Teamspace Member"
        verbose_name_plural = "Teamspace Members"
        db_table = "teamspace_members"
        ordering = ("-created_at",)

    def __str__(self):
        return f"{self.member.email} <{self.teamspace.name}>"
