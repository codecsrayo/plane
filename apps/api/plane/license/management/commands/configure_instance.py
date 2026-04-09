# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

# Python imports
import os

# Django imports
from django.core.management.base import BaseCommand, CommandError

# Module imports
from plane.license.models import InstanceConfiguration
from plane.utils.instance_config_variables import instance_config_variables


class Command(BaseCommand):
    help = "Configure instance variables"

    def handle(self, *args, **options):
        from plane.license.utils.encryption import encrypt_data
        from plane.license.utils.instance_value import get_configuration_value

        mandatory_keys = ["SECRET_KEY"]

        for item in mandatory_keys:
            if not os.environ.get(item):
                raise CommandError(f"{item} env variable is required.")

        for item in instance_config_variables:
            obj, created = InstanceConfiguration.objects.get_or_create(key=item.get("key"))
            if created:
                obj.category = item.get("category")
                obj.is_encrypted = item.get("is_encrypted", False)
                # Determine value: prioritize env, then current config (if exists), then item default
                value = item.get("value")

                # Specific logic for authentication toggles if they are being created for the first time
                # to maintain backward compatibility with environment-based auto-enabling
                if item.get("key") == "IS_GOOGLE_ENABLED":
                    GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET = get_configuration_value(
                        [
                            {
                                "key": "GOOGLE_CLIENT_ID",
                                "default": "",
                            },
                            {
                                "key": "GOOGLE_CLIENT_SECRET",
                                "default": "",
                            },
                        ]
                    )
                    if bool(GOOGLE_CLIENT_ID) and bool(GOOGLE_CLIENT_SECRET):
                        value = "1"
                elif item.get("key") == "IS_GITHUB_ENABLED":
                    GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET = get_configuration_value(
                        [
                            {
                                "key": "GITHUB_CLIENT_ID",
                                "default": "",
                            },
                            {
                                "key": "GITHUB_CLIENT_SECRET",
                                "default": "",
                            },
                        ]
                    )
                    if bool(GITHUB_CLIENT_ID) and bool(GITHUB_CLIENT_SECRET):
                        value = "1"
                elif item.get("key") == "IS_GITHUB_INTEGRATION_ENABLED":
                    GITHUB_APP_ID, GITHUB_APP_NAME = get_configuration_value(
                        [
                            {
                                "key": "GITHUB_APP_ID",
                                "default": "",
                            },
                            {
                                "key": "GITHUB_APP_NAME",
                                "default": "",
                            },
                        ]
                    )
                    if bool(GITHUB_APP_ID) and bool(GITHUB_APP_NAME):
                        value = "1"
                elif item.get("key") == "IS_GITLAB_ENABLED":
                    GITLAB_HOST, GITLAB_CLIENT_ID, GITLAB_CLIENT_SECRET = get_configuration_value(
                        [
                            {
                                "key": "GITLAB_HOST",
                                "default": "https://gitlab.com",
                            },
                            {
                                "key": "GITLAB_CLIENT_ID",
                                "default": "",
                            },
                            {
                                "key": "GITLAB_CLIENT_SECRET",
                                "default": "",
                            },
                        ]
                    )
                    if bool(GITLAB_HOST) and bool(GITLAB_CLIENT_ID) and bool(GITLAB_CLIENT_SECRET):
                        value = "1"
                elif item.get("key") == "IS_GITLAB_INTEGRATION_ENABLED":
                    GITLAB_CLIENT_ID_CHECK, GITLAB_CLIENT_SECRET_CHECK = get_configuration_value(
                        [
                            {
                                "key": "GITLAB_CLIENT_ID",
                                "default": "",
                            },
                            {
                                "key": "GITLAB_CLIENT_SECRET",
                                "default": "",
                            },
                        ]
                    )
                    if bool(GITLAB_CLIENT_ID_CHECK) and bool(GITLAB_CLIENT_SECRET_CHECK):
                        value = "1"
                elif item.get("key") == "IS_GITEA_ENABLED":
                    GITEA_HOST, GITEA_CLIENT_ID, GITEA_CLIENT_SECRET = get_configuration_value(
                        [
                            {
                                "key": "GITEA_HOST",
                                "default": "",
                            },
                            {
                                "key": "GITEA_CLIENT_ID",
                                "default": "",
                            },
                            {
                                "key": "GITEA_CLIENT_SECRET",
                                "default": "",
                            },
                        ]
                    )
                    if bool(GITEA_HOST) and bool(GITEA_CLIENT_ID) and bool(GITEA_CLIENT_SECRET):
                        value = "1"
                elif item.get("key") == "IS_SLACK_ENABLED":
                    SLACK_CLIENT_ID, SLACK_CLIENT_SECRET = get_configuration_value(
                        [
                            {
                                "key": "SLACK_CLIENT_ID",
                                "default": "",
                            },
                            {
                                "key": "SLACK_CLIENT_SECRET",
                                "default": "",
                            },
                        ]
                    )
                    if bool(SLACK_CLIENT_ID) and bool(SLACK_CLIENT_SECRET):
                        value = "1"

                if item.get("is_encrypted", False):
                    obj.value = encrypt_data(value)
                else:
                    obj.value = value
                obj.save()
                self.stdout.write(self.style.SUCCESS(f"{obj.key} loaded with value."))
            else:
                self.stdout.write(self.style.WARNING(f"{obj.key} configuration already exists"))
