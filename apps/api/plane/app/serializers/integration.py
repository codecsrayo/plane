# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

# Module imports
from .base import BaseSerializer
from plane.db.models import Integration, WorkspaceIntegration


class IntegrationSerializer(BaseSerializer):
    class Meta:
        model = Integration
        fields = "__all__"


class WorkspaceIntegrationSerializer(BaseSerializer):
    integration_detail = IntegrationSerializer(source="integration", read_only=True)

    class Meta:
        model = WorkspaceIntegration
        fields = "__all__"
