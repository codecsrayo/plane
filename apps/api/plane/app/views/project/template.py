# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

# Django imports
from django.db.models import Exists, OuterRef, Prefetch, Subquery

# Third party imports
from rest_framework import status
from rest_framework.response import Response

# Module imports
from plane.app.permissions import allow_permission, ROLE
from plane.app.serializers import ProjectTemplateSerializer, ProjectSerializer, ProjectListSerializer
from plane.app.views.base import BaseViewSet, BaseAPIView
from plane.db.models import (
    DEFAULT_STATES,
    Cycle,
    DeployBoard,
    Label,
    Module,
    Project,
    ProjectMember,
    ProjectTemplate,
    ProjectUserProperty,
    State,
    UserFavorite,
    Workspace,
)
from plane.bgtasks.webhook_task import model_activity
from plane.utils.host import base_host


class ProjectTemplateViewSet(BaseViewSet):
    serializer_class = ProjectTemplateSerializer
    model = ProjectTemplate

    def get_queryset(self):
        return ProjectTemplate.objects.filter(workspace__slug=self.kwargs.get("slug"))

    @allow_permission(allowed_roles=[ROLE.ADMIN, ROLE.MEMBER, ROLE.GUEST], level="WORKSPACE")
    def list(self, request, slug):
        templates = self.get_queryset()
        serializer = ProjectTemplateSerializer(templates, many=True)
        return Response(serializer.data, status=status.HTTP_200_OK)

    @allow_permission(allowed_roles=[ROLE.ADMIN, ROLE.MEMBER], level="WORKSPACE")
    def create(self, request, slug):
        workspace = Workspace.objects.get(slug=slug)
        serializer = ProjectTemplateSerializer(data=request.data)
        if serializer.is_valid():
            serializer.save(workspace=workspace)
            return Response(serializer.data, status=status.HTTP_201_CREATED)
        return Response(serializer.errors, status=status.HTTP_400_BAD_REQUEST)

    @allow_permission(allowed_roles=[ROLE.ADMIN, ROLE.MEMBER, ROLE.GUEST], level="WORKSPACE")
    def retrieve(self, request, slug, pk):
        template = self.get_queryset().filter(pk=pk).first()
        if not template:
            return Response({"error": "Template not found"}, status=status.HTTP_404_NOT_FOUND)
        serializer = ProjectTemplateSerializer(template)
        return Response(serializer.data, status=status.HTTP_200_OK)

    @allow_permission(allowed_roles=[ROLE.ADMIN, ROLE.MEMBER], level="WORKSPACE")
    def partial_update(self, request, slug, pk):
        template = self.get_queryset().filter(pk=pk).first()
        if not template:
            return Response({"error": "Template not found"}, status=status.HTTP_404_NOT_FOUND)
        serializer = ProjectTemplateSerializer(template, data=request.data, partial=True)
        if serializer.is_valid():
            serializer.save()
            return Response(serializer.data, status=status.HTTP_200_OK)
        return Response(serializer.errors, status=status.HTTP_400_BAD_REQUEST)

    @allow_permission(allowed_roles=[ROLE.ADMIN], level="WORKSPACE")
    def destroy(self, request, slug, pk):
        template = self.get_queryset().filter(pk=pk).first()
        if not template:
            return Response({"error": "Template not found"}, status=status.HTTP_404_NOT_FOUND)
        template.delete()
        return Response(status=status.HTTP_204_NO_CONTENT)


class ProjectSaveAsTemplateEndpoint(BaseAPIView):
    """Snapshot an existing project as a new template."""

    @allow_permission(allowed_roles=[ROLE.ADMIN, ROLE.MEMBER], level="PROJECT")
    def post(self, request, slug, project_id):
        project = Project.objects.filter(workspace__slug=slug, pk=project_id).first()
        if not project:
            return Response({"error": "Project not found"}, status=status.HTTP_404_NOT_FOUND)

        states = list(
            State.objects.filter(project=project).values(
                "name", "color", "sequence", "group", "default", "description"
            )
        )
        labels = list(
            Label.objects.filter(project=project).values("name", "color", "description", "sort_order")
        )
        modules = list(
            Module.objects.filter(project=project).values("name", "description", "status")
        )
        cycles = list(
            Cycle.objects.filter(project=project).values("name", "description")
        )
        members = list(
            ProjectMember.objects.filter(project=project, is_active=True).values("role")
        )

        template_data = {
            "states": states,
            "labels": labels,
            "modules": modules,
            "cycles": cycles,
            "member_roles": members,
        }

        workspace = project.workspace
        name = request.data.get("name", f"{project.name} Template")
        description = request.data.get("description", "")

        template = ProjectTemplate.objects.create(
            workspace=workspace,
            name=name,
            description=description,
            template_data=template_data,
            created_by=request.user,
        )
        serializer = ProjectTemplateSerializer(template)
        return Response(serializer.data, status=status.HTTP_201_CREATED)


class ProjectTemplateInstantiateEndpoint(BaseAPIView):
    """Create a new project from a template."""

    @allow_permission(allowed_roles=[ROLE.ADMIN, ROLE.MEMBER], level="WORKSPACE")
    def post(self, request, slug, template_id):
        template = ProjectTemplate.objects.filter(workspace__slug=slug, pk=template_id).first()
        if not template:
            return Response({"error": "Template not found"}, status=status.HTTP_404_NOT_FOUND)

        workspace = Workspace.objects.get(slug=slug)
        serializer = ProjectSerializer(
            data=request.data,
            context={"workspace_id": workspace.id},
        )
        if not serializer.is_valid():
            return Response(serializer.errors, status=status.HTTP_400_BAD_REQUEST)

        project = serializer.save()

        ProjectMember.objects.create(
            project=project,
            member=request.user,
            role=ROLE.ADMIN.value,
        )

        tdata = template.template_data

        if tdata.get("states"):
            State.objects.bulk_create(
                [
                    State(
                        name=s["name"],
                        color=s["color"],
                        sequence=s.get("sequence", 65535),
                        group=s.get("group", "backlog"),
                        default=s.get("default", False),
                        description=s.get("description", ""),
                        project=project,
                        workspace=workspace,
                        created_by=request.user,
                    )
                    for s in tdata["states"]
                ]
            )
        else:
            State.objects.bulk_create(
                [
                    State(
                        name=s["name"],
                        color=s["color"],
                        sequence=s["sequence"],
                        group=s["group"],
                        default=s.get("default", False),
                        project=project,
                        workspace=workspace,
                        created_by=request.user,
                    )
                    for s in DEFAULT_STATES
                ]
            )

        if tdata.get("labels"):
            Label.objects.bulk_create(
                [
                    Label(
                        name=l["name"],
                        color=l.get("color", ""),
                        description=l.get("description", ""),
                        sort_order=l.get("sort_order", 65535),
                        project=project,
                        workspace=workspace,
                        created_by=request.user,
                    )
                    for l in tdata["labels"]
                ]
            )

        if tdata.get("modules"):
            Module.objects.bulk_create(
                [
                    Module(
                        name=m["name"],
                        description=m.get("description", ""),
                        status=m.get("status", "backlog"),
                        project=project,
                        workspace=workspace,
                        created_by=request.user,
                    )
                    for m in tdata["modules"]
                ]
            )

        # Cycles have no date fields from template (those are project-specific)
        if tdata.get("cycles"):
            Cycle.objects.bulk_create(
                [
                    Cycle(
                        name=c["name"],
                        description=c.get("description", ""),
                        owned_by=request.user,
                        project=project,
                        workspace=workspace,
                        created_by=request.user,
                    )
                    for c in tdata["cycles"]
                ]
            )

        model_activity.delay(
            model_name="project",
            model_id=str(project.id),
            requested_data=request.data,
            current_instance=None,
            actor_id=request.user.id,
            slug=slug,
            origin=base_host(request=request, is_app=True),
        )

        sort_order = ProjectUserProperty.objects.filter(
            user=request.user,
            project_id=OuterRef("pk"),
            workspace__slug=slug,
        ).values("sort_order")

        result = (
            Project.objects.filter(pk=project.id)
            .annotate(
                is_favorite=Exists(
                    UserFavorite.objects.filter(
                        user=request.user,
                        entity_identifier=OuterRef("pk"),
                        entity_type="project",
                        project_id=OuterRef("pk"),
                    )
                )
            )
            .annotate(
                member_role=ProjectMember.objects.filter(
                    project_id=OuterRef("pk"),
                    member_id=request.user.id,
                    is_active=True,
                ).values("role")
            )
            .annotate(
                anchor=DeployBoard.objects.filter(
                    entity_name="project",
                    entity_identifier=OuterRef("pk"),
                    workspace__slug=slug,
                ).values("anchor")
            )
            .annotate(sort_order=Subquery(sort_order))
            .prefetch_related(
                Prefetch(
                    "project_projectmember",
                    queryset=ProjectMember.objects.filter(
                        workspace__slug=slug, is_active=True
                    ).select_related("member"),
                    to_attr="members_list",
                )
            )
            .first()
        )

        return Response(ProjectListSerializer(result).data, status=status.HTTP_201_CREATED)
