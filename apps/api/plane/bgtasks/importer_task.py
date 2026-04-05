# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

import requests
from celery import shared_task
from plane.db.models import (
    Importer, Issue, Label, State, IssueComment, IssueLabel,
    GithubRepositorySync, GithubIssueSync, GithubCommentSync,
    GitlabRepositorySync, GitlabIssueSync, GitlabCommentSync
)


@shared_task
def github_importer_task(importer_id: str):
    """Import issues from a GitHub repository into a Plane project"""
    try:
        importer = Importer.objects.get(id=importer_id)
        importer.status = "processing"
        importer.save()

        config = importer.config
        metadata = importer.metadata
        github_token = config.get("github_token")
        owner = metadata.get("owner")
        repo = metadata.get("name")
        project = importer.project

        headers = {
            "Authorization": f"Bearer {github_token}",
            "Accept": "application/vnd.github+json",
        }

        # Project default state
        default_state = State.objects.filter(
            project=project, default=True
        ).first() or State.objects.filter(project=project).first()

        # Import labels if configured
        label_map = {}
        if config.get("import_labels", True):
            gh_labels_response = requests.get(
                f"https://api.github.com/repos/{owner}/{repo}/labels",
                headers=headers,
            )
            if gh_labels_response.status_code == 200:
                for gh_label in gh_labels_response.json():
                    label, _ = Label.objects.get_or_create(
                        project=project,
                        workspace=project.workspace,
                        name=gh_label["name"],
                        defaults={"color": f"#{gh_label.get('color', 'cccccc')}"},
                    )
                    label_map[gh_label["name"]] = label

        # Paginate issues
        page = 1
        imported_count = 0

        while True:
            response = requests.get(
                f"https://api.github.com/repos/{owner}/{repo}/issues",
                headers=headers,
                params={
                    "state": "all",
                    "per_page": 50,
                    "page": page,
                    "sort": "created",
                    "direction": "asc",
                },
            )

            if response.status_code != 200:
                break

            issues = response.json()
            if not issues:
                break

            for gh_issue in issues:
                # Skip pull requests
                if "pull_request" in gh_issue:
                    continue

                # Create Issue in Plane
                issue = Issue.objects.create(
                    project=project,
                    workspace=project.workspace,
                    name=gh_issue["title"][:255],
                    description_html=gh_issue.get("body") or "",
                    state=default_state,
                    created_by=importer.initiated_by,
                    updated_by=importer.initiated_by,
                )

                # Create GitHub sync record for the issue
                repo_sync = GithubRepositorySync.objects.filter(project=project).first()
                if repo_sync:
                    GithubIssueSync.objects.create(
                        issue=issue,
                        repository_sync=repo_sync,
                        github_issue_id=gh_issue["id"],
                        repo_issue_id=gh_issue["number"],
                        issue_url=gh_issue["html_url"],
                        project=project,
                        workspace=project.workspace
                    )

                # Assign labels
                for gh_label in gh_issue.get("labels", []):
                    if gh_label["name"] in label_map:
                        IssueLabel.objects.get_or_create(
                            issue=issue,
                            label=label_map[gh_label["name"]],
                            project=project,
                            workspace=project.workspace,
                        )

                # Import comments if configured
                if config.get("import_comments", True) and gh_issue.get("comments", 0) > 0:
                    comments_response = requests.get(
                        f"https://api.github.com/repos/{owner}/{repo}/issues/{gh_issue['number']}/comments",
                        headers=headers,
                    )
                    if comments_response.status_code == 200:
                        for comment in comments_response.json():
                            new_comment = IssueComment.objects.create(
                                issue=issue,
                                project=project,
                                workspace=project.workspace,
                                comment_html=comment.get("body", ""),
                                created_by=importer.initiated_by,
                                updated_by=importer.initiated_by,
                            )
                            # Create comment sync record
                            if repo_sync:
                                github_issue_sync = GithubIssueSync.objects.filter(issue=issue).first()
                                if github_issue_sync:
                                    GithubCommentSync.objects.create(
                                        comment=new_comment,
                                        issue_sync=github_issue_sync,
                                        repo_comment_id=comment["id"],
                                        project=project,
                                        workspace=project.workspace
                                    )

                imported_count += 1

            page += 1

        # Mark as completed
        importer.status = "completed"
        importer.imported_data = {"imported_count": imported_count}
        importer.save()

    except Exception as e:
        importer.status = "failed"
        importer.imported_data = {"error": str(e)}
        importer.save()
        raise


@shared_task
def gitlab_importer_task(importer_id: str):
    """Import issues from a GitLab project into a Plane project"""
    try:
        importer = Importer.objects.get(id=importer_id)
        importer.status = "processing"
        importer.save()

        config = importer.config
        metadata = importer.metadata
        gitlab_token = config.get("gitlab_token")
        gitlab_host = config.get("gitlab_host", "https://gitlab.com").rstrip("/")
        gitlab_project_id = metadata.get("gitlab_project_id")
        project = importer.project

        headers = {
            "PRIVATE-TOKEN": gitlab_token,
        }

        # Project default state
        default_state = State.objects.filter(
            project=project, default=True
        ).first() or State.objects.filter(project=project).first()

        # Import labels
        label_map = {}
        if config.get("import_labels", True):
            gl_labels_response = requests.get(
                f"{gitlab_host}/api/v4/projects/{gitlab_project_id}/labels",
                headers=headers,
            )
            if gl_labels_response.status_code == 200:
                for gl_label in gl_labels_response.json():
                    label, _ = Label.objects.get_or_create(
                        project=project,
                        workspace=project.workspace,
                        name=gl_label["name"],
                        defaults={"color": gl_label.get("color", "#cccccc")},
                    )
                    label_map[gl_label["name"]] = label

        # Paginate issues
        page = 1
        imported_count = 0

        while True:
            response = requests.get(
                f"{gitlab_host}/api/v4/projects/{gitlab_project_id}/issues",
                headers=headers,
                params={
                    "page": page,
                    "per_page": 50,
                    "order_by": "created_at",
                    "sort": "asc",
                },
            )

            if response.status_code != 200:
                break

            issues = response.json()
            if not issues:
                break

            for gl_issue in issues:
                # Create Issue in Plane
                issue = Issue.objects.create(
                    project=project,
                    workspace=project.workspace,
                    name=gl_issue["title"][:255],
                    description_html=gl_issue.get("description") or "",
                    state=default_state,
                    created_by=importer.initiated_by,
                    updated_by=importer.initiated_by,
                )

                # Create GitLab sync record for the issue
                repo_sync = GitlabRepositorySync.objects.filter(project=project).first()
                if repo_sync:
                    GitlabIssueSync.objects.create(
                        issue=issue,
                        repository_sync=repo_sync,
                        gitlab_issue_id=gl_issue["id"],
                        repo_issue_id=gl_issue["iid"],
                        issue_url=gl_issue["web_url"],
                        project=project,
                        workspace=project.workspace
                    )

                # Assign labels
                for gl_label_name in gl_issue.get("labels", []):
                    if gl_label_name in label_map:
                        IssueLabel.objects.get_or_create(
                            issue=issue,
                            label=label_map[gl_label_name],
                            project=project,
                            workspace=project.workspace,
                        )

                # Import comments (notes in GitLab)
                if config.get("import_comments", True) and gl_issue.get("user_notes_count", 0) > 0:
                    notes_response = requests.get(
                        f"{gitlab_host}/api/v4/projects/{gitlab_project_id}/issues/{gl_issue['iid']}/notes",
                        headers=headers,
                    )
                    if notes_response.status_code == 200:
                        for note in notes_response.json():
                            # Skip system notes
                            if note.get("system", False):
                                continue

                            new_note = IssueComment.objects.create(
                                issue=issue,
                                project=project,
                                workspace=project.workspace,
                                comment_html=note.get("body", ""),
                                created_by=importer.initiated_by,
                                updated_by=importer.initiated_by,
                            )
                            # Create comment sync record
                            if repo_sync:
                                gitlab_issue_sync = GitlabIssueSync.objects.filter(issue=issue).first()
                                if gitlab_issue_sync:
                                    GitlabCommentSync.objects.create(
                                        comment=new_note,
                                        issue_sync=gitlab_issue_sync,
                                        repo_comment_id=note["id"],
                                        project=project,
                                        workspace=project.workspace
                                    )

                imported_count += 1

            page += 1

        # Mark as completed
        importer.status = "completed"
        importer.imported_data = {"imported_count": imported_count}
        importer.save()

    except Exception as e:
        importer.status = "failed"
        importer.imported_data = {"error": str(e)}
        importer.save()
        raise
