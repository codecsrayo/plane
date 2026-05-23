# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

# Third party imports
from rest_framework import serializers

# Module imports
from plane.db.models import Teamspace, TeamspaceMember, WorkspaceMember
from .base import BaseSerializer


VALID_ROLES = (20, 15, 5)


class TeamspaceSerializer(BaseSerializer):
    class Meta:
        model = Teamspace
        fields = ["id", "name", "description", "workspace", "logo_props", "created_at", "updated_at"]
        read_only_fields = ["id", "workspace", "created_at", "updated_at"]


class TeamspaceCreateSerializer(BaseSerializer):
    class Meta:
        model = Teamspace
        fields = ["id", "name", "description", "logo_props"]
        read_only_fields = ["id"]


class TeamspaceUpdateSerializer(BaseSerializer):
    class Meta:
        model = Teamspace
        fields = ["name", "description", "logo_props"]


class TeamspaceMemberSerializer(BaseSerializer):
    member = serializers.PrimaryKeyRelatedField(read_only=True)

    class Meta:
        model = TeamspaceMember
        fields = ["id", "teamspace", "member", "role", "created_at", "updated_at"]
        read_only_fields = ["id", "teamspace", "created_at", "updated_at"]


class TeamspaceMemberCreateSerializer(BaseSerializer):
    member = serializers.UUIDField()

    def validate_member(self, value):
        slug = self.context.get("slug")
        if not slug:
            raise serializers.ValidationError("Workspace slug is required", code="INVALID_SLUG")
        if not WorkspaceMember.objects.filter(workspace__slug=slug, member_id=value, is_active=True).exists():
            raise serializers.ValidationError("User is not an active member of this workspace", code="INVALID_MEMBER")
        return value

    def validate_role(self, value):
        if value not in VALID_ROLES:
            raise serializers.ValidationError("Invalid role", code="INVALID_ROLE")
        return value

    class Meta:
        model = TeamspaceMember
        fields = ["id", "member", "role"]
        read_only_fields = ["id"]


class TeamspaceMemberUpdateSerializer(BaseSerializer):
    def validate_role(self, value):
        if value not in VALID_ROLES:
            raise serializers.ValidationError("Invalid role", code="INVALID_ROLE")
        return value

    class Meta:
        model = TeamspaceMember
        fields = ["role"]
