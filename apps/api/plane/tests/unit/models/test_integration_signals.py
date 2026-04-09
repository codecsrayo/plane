# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

from unittest.mock import Mock

import pytest

from plane.db.models import Issue, IssueComment, Project, State, Workspace
from plane.db.models.integration.github import GithubIssueSync
from plane.db.models.integration.gitlab import GitlabIssueSync
from plane.db.models.integration import signals


@pytest.fixture
def workspace(create_user):
    return Workspace.objects.create(
        name="Test Workspace",
        slug="test-workspace-signals",
        owner=create_user,
    )


@pytest.fixture
def project(workspace, create_user):
    return Project.objects.create(
        name="Test Project",
        identifier="TPS",
        workspace=workspace,
        created_by=create_user,
    )


@pytest.fixture
def state(project):
    return State.objects.create(
        name="Todo",
        project=project,
        group="backlog",
        default=True,
    )


@pytest.fixture
def issue(workspace, project, state, create_user):
    return Issue.objects.create(
        name="Test Issue",
        workspace=workspace,
        project=project,
        state=state,
        created_by=create_user,
        updated_by=create_user,
    )


@pytest.mark.unit
class TestIntegrationSignals:
    @pytest.mark.django_db
    def test_issue_update_skips_gitlab_sync_when_table_is_missing(self, issue, monkeypatch):
        github_delay = Mock()
        gitlab_delay = Mock()

        monkeypatch.setattr(signals.sync_issue_to_github_task, "delay", github_delay)
        monkeypatch.setattr(signals.sync_issue_to_gitlab_task, "delay", gitlab_delay)
        monkeypatch.setattr(
            signals,
            "_table_exists",
            lambda using, table_name: table_name != GitlabIssueSync._meta.db_table,
        )

        issue.name = "Updated Issue"
        issue.save()

        issue.refresh_from_db()
        assert issue.name == "Updated Issue"
        github_delay.assert_not_called()
        gitlab_delay.assert_not_called()

    @pytest.mark.django_db
    def test_comment_create_skips_gitlab_sync_when_table_is_missing(self, workspace, project, issue, create_user, monkeypatch):
        github_delay = Mock()
        gitlab_delay = Mock()

        monkeypatch.setattr(signals.sync_comment_to_github_task, "delay", github_delay)
        monkeypatch.setattr(signals.sync_comment_to_gitlab_task, "delay", gitlab_delay)
        monkeypatch.setattr(
            signals,
            "_table_exists",
            lambda using, table_name: table_name != GitlabIssueSync._meta.db_table,
        )

        comment = IssueComment.objects.create(
            workspace=workspace,
            project=project,
            issue=issue,
            comment_html="<p>Test comment</p>",
            comment_json={"type": "doc", "content": [{"type": "paragraph", "text": "Test comment"}]},
            created_by=create_user,
            updated_by=create_user,
        )

        assert comment.id is not None
        github_delay.assert_not_called()
        gitlab_delay.assert_not_called()
        assert GithubIssueSync.objects.filter(issue=issue).exists() is False
