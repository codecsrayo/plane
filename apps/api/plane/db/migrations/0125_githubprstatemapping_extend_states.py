# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ("db", "0124_githubprstatemapping_usergithubconnection"),
    ]

    operations = [
        # Extend github_pr_state choices and bump max_length to 20 (already 20, no DB change needed for length)
        migrations.AlterField(
            model_name="githubprstatemapping",
            name="github_pr_state",
            field=models.CharField(
                choices=[
                    ("draft_open", "Draft Open"),
                    ("open", "Open"),
                    ("review_requested", "Review Requested"),
                    ("ready_for_merge", "Ready for Merge"),
                    ("merged", "Merged"),
                    ("closed", "Closed"),
                ],
                max_length=20,
            ),
        ),
        # Add prevent_regression field
        migrations.AddField(
            model_name="githubprstatemapping",
            name="prevent_regression",
            field=models.BooleanField(
                default=False,
                help_text="Prevent issues from moving to an earlier state due to PR updates",
            ),
        ),
    ]
