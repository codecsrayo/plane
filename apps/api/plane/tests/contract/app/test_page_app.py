# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

import pytest
from rest_framework import status

from plane.db.models import Page, Project, ProjectMember, ProjectPage


class TestPageBase:
    def get_pages_url(self, workspace_slug, project_id, page_id=None):
        base_url = f"/api/workspaces/{workspace_slug}/projects/{project_id}/pages/"
        if page_id:
            return f"{base_url}{page_id}/"
        return base_url


@pytest.mark.contract
class TestPageFeatureGate(TestPageBase):
    @pytest.mark.django_db
    def test_pages_list_forbidden_when_page_view_disabled(self, session_client, workspace, create_user):
        project = Project.objects.create(name="Test Project", identifier="TP", workspace=workspace, page_view=False)
        ProjectMember.objects.create(project=project, member=create_user, role=20, is_active=True)

        response = session_client.get(self.get_pages_url(workspace.slug, project.id))

        assert response.status_code == status.HTTP_403_FORBIDDEN

    @pytest.mark.django_db
    def test_page_detail_forbidden_when_page_view_disabled(self, session_client, workspace, create_user):
        project = Project.objects.create(name="Test Project", identifier="TP", workspace=workspace, page_view=False)
        ProjectMember.objects.create(project=project, member=create_user, role=20, is_active=True)
        page = Page.objects.create(workspace=workspace, name="Hidden page", owned_by=create_user)
        ProjectPage.objects.create(workspace=workspace, project=project, page=page)

        response = session_client.get(self.get_pages_url(workspace.slug, project.id, page.id))

        assert response.status_code == status.HTTP_403_FORBIDDEN
