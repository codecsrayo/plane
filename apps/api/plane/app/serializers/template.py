# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

from .base import BaseSerializer
from plane.db.models import ProjectTemplate


class ProjectTemplateSerializer(BaseSerializer):
    class Meta:
        model = ProjectTemplate
        fields = [
            "id",
            "workspace_id",
            "name",
            "description",
            "template_data",
            "created_at",
            "updated_at",
            "created_by_id",
            "updated_by_id",
        ]
        read_only_fields = ["workspace", "created_at", "updated_at", "created_by", "updated_by"]
