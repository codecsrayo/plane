# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

from django.db import migrations


def add_slack_integration(apps, schema_editor):
    Integration = apps.get_model("db", "Integration")

    Integration.objects.get_or_create(
        provider="slack",
        defaults={
            "title": "Slack",
            "network": 2,  # Public
            "verified": True,
        },
    )


def remove_slack_integration(apps, schema_editor):
    Integration = apps.get_model("db", "Integration")
    Integration.objects.filter(provider="slack").delete()


class Migration(migrations.Migration):

    dependencies = [
        ("db", "0122_add_github_gitlab_integrations"),
    ]

    operations = [
        migrations.RunPython(add_slack_integration, reverse_code=remove_slack_integration),
    ]
