# Generated migration for Teamspace and TeamspaceMember models

import uuid

import django.db.models.deletion
from django.conf import settings
from django.db import migrations, models


class Migration(migrations.Migration):
    dependencies = [
        ("db", "0127_projecttemplate"),
    ]

    operations = [
        migrations.CreateModel(
            name="Teamspace",
            fields=[
                ("created_at", models.DateTimeField(auto_now_add=True, verbose_name="Created At")),
                ("updated_at", models.DateTimeField(auto_now=True, verbose_name="Last Modified At")),
                ("deleted_at", models.DateTimeField(null=True, blank=True)),
                (
                    "id",
                    models.UUIDField(
                        db_index=True,
                        default=uuid.uuid4,
                        editable=False,
                        primary_key=True,
                        serialize=False,
                        unique=True,
                    ),
                ),
                (
                    "created_by",
                    models.ForeignKey(
                        null=True,
                        on_delete=django.db.models.deletion.SET_NULL,
                        related_name="%(class)s_created_by",
                        to=settings.AUTH_USER_MODEL,
                        verbose_name="Created By",
                    ),
                ),
                (
                    "updated_by",
                    models.ForeignKey(
                        null=True,
                        on_delete=django.db.models.deletion.SET_NULL,
                        related_name="%(class)s_updated_by",
                        to=settings.AUTH_USER_MODEL,
                        verbose_name="Last Modified By",
                    ),
                ),
                (
                    "workspace",
                    models.ForeignKey(
                        on_delete=django.db.models.deletion.CASCADE,
                        related_name="workspace_teamspaces",
                        to="db.workspace",
                    ),
                ),
                ("name", models.CharField(max_length=255, verbose_name="Teamspace Name")),
                ("description", models.TextField(blank=True, verbose_name="Teamspace Description")),
                ("logo_props", models.JSONField(default=dict)),
            ],
            options={
                "verbose_name": "Teamspace",
                "verbose_name_plural": "Teamspaces",
                "db_table": "teamspaces",
                "ordering": ("-created_at",),
            },
        ),
        migrations.AddConstraint(
            model_name="teamspace",
            constraint=models.UniqueConstraint(
                condition=models.Q(deleted_at__isnull=True),
                fields=["name", "workspace"],
                name="teamspace_unique_name_workspace_when_deleted_at_null",
            ),
        ),
        migrations.AlterUniqueTogether(
            name="teamspace",
            unique_together={("name", "workspace", "deleted_at")},
        ),
        migrations.CreateModel(
            name="TeamspaceMember",
            fields=[
                ("created_at", models.DateTimeField(auto_now_add=True, verbose_name="Created At")),
                ("updated_at", models.DateTimeField(auto_now=True, verbose_name="Last Modified At")),
                ("deleted_at", models.DateTimeField(null=True, blank=True)),
                (
                    "id",
                    models.UUIDField(
                        db_index=True,
                        default=uuid.uuid4,
                        editable=False,
                        primary_key=True,
                        serialize=False,
                        unique=True,
                    ),
                ),
                (
                    "created_by",
                    models.ForeignKey(
                        null=True,
                        on_delete=django.db.models.deletion.SET_NULL,
                        related_name="%(class)s_created_by",
                        to=settings.AUTH_USER_MODEL,
                        verbose_name="Created By",
                    ),
                ),
                (
                    "updated_by",
                    models.ForeignKey(
                        null=True,
                        on_delete=django.db.models.deletion.SET_NULL,
                        related_name="%(class)s_updated_by",
                        to=settings.AUTH_USER_MODEL,
                        verbose_name="Last Modified By",
                    ),
                ),
                (
                    "teamspace",
                    models.ForeignKey(
                        on_delete=django.db.models.deletion.CASCADE,
                        related_name="teamspace_members",
                        to="db.teamspace",
                    ),
                ),
                (
                    "member",
                    models.ForeignKey(
                        on_delete=django.db.models.deletion.CASCADE,
                        related_name="member_teamspaces",
                        to=settings.AUTH_USER_MODEL,
                    ),
                ),
                (
                    "role",
                    models.PositiveSmallIntegerField(
                        choices=[(20, "Admin"), (15, "Member"), (5, "Guest")],
                        default=15,
                    ),
                ),
            ],
            options={
                "verbose_name": "Teamspace Member",
                "verbose_name_plural": "Teamspace Members",
                "db_table": "teamspace_members",
                "ordering": ("-created_at",),
            },
        ),
        migrations.AddConstraint(
            model_name="teamspacemember",
            constraint=models.UniqueConstraint(
                condition=models.Q(deleted_at__isnull=True),
                fields=["teamspace", "member"],
                name="teamspace_member_unique_teamspace_member_when_deleted_at_null",
            ),
        ),
        migrations.AlterUniqueTogether(
            name="teamspacemember",
            unique_together={("teamspace", "member", "deleted_at")},
        ),
    ]
