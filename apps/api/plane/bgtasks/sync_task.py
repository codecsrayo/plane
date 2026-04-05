# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

import requests
from celery import shared_task
from plane.db.models import (
    Issue, IssueComment, GithubIssueSync, GitlabIssueSync, GithubCommentSync, GitlabCommentSync
)

@shared_task
def sync_issue_to_github_task(issue_id):
    issue = Issue.objects.get(id=issue_id)
    github_sync = GithubIssueSync.objects.filter(issue=issue).first()
    if github_sync:
        repo_sync = github_sync.repository_sync
        token = repo_sync.credentials.get("token")
        if token:
            owner = repo_sync.repository.owner
            repo = repo_sync.repository.name
            issue_number = github_sync.repo_issue_id

            requests.patch(
                f"https://api.github.com/repos/{owner}/{repo}/issues/{issue_number}",
                headers={
                    "Authorization": f"token {token}",
                    "Accept": "application/vnd.github+json",
                },
                json={
                    "title": issue.name,
                    "body": issue.description_html,
                }
            )

@shared_task
def sync_issue_to_gitlab_task(issue_id):
    issue = Issue.objects.get(id=issue_id)
    gitlab_sync = GitlabIssueSync.objects.filter(issue=issue).first()
    if gitlab_sync:
        repo_sync = gitlab_sync.repository_sync
        token = repo_sync.credentials.get("token")
        host = repo_sync.credentials.get("host", "https://gitlab.com").rstrip("/")
        if token:
            project_id = repo_sync.repository.repository_id
            issue_iid = gitlab_sync.repo_issue_id

            requests.put(
                f"{host}/api/v4/projects/{project_id}/issues/{issue_iid}",
                headers={"PRIVATE-TOKEN": token},
                json={
                    "title": issue.name,
                    "description": issue.description_html,
                }
            )

@shared_task
def sync_comment_to_github_task(comment_id):
    comment = IssueComment.objects.get(id=comment_id)
    github_issue_sync = GithubIssueSync.objects.filter(issue=comment.issue).first()
    if github_issue_sync:
        repo_sync = github_issue_sync.repository_sync
        token = repo_sync.credentials.get("token")
        if token:
            owner = repo_sync.repository.owner
            repo = repo_sync.repository.name
            issue_number = github_issue_sync.repo_issue_id

            github_comment_sync = GithubCommentSync.objects.filter(comment=comment).first()
            if github_comment_sync:
                # Update
                requests.patch(
                    f"https://api.github.com/repos/{owner}/{repo}/issues/comments/{github_comment_sync.repo_comment_id}",
                    headers={
                        "Authorization": f"token {token}",
                        "Accept": "application/vnd.github+json",
                    },
                    json={"body": comment.comment_html}
                )
            else:
                # Create
                response = requests.post(
                    f"https://api.github.com/repos/{owner}/{repo}/issues/{issue_number}/comments",
                    headers={
                        "Authorization": f"token {token}",
                        "Accept": "application/vnd.github+json",
                    },
                    json={"body": comment.comment_html}
                )
                if response.status_code == 201:
                    data = response.json()
                    GithubCommentSync.objects.create(
                        comment=comment,
                        issue_sync=github_issue_sync,
                        repo_comment_id=data["id"],
                        project=comment.project,
                        workspace=comment.workspace
                    )

@shared_task
def sync_comment_to_gitlab_task(comment_id):
    comment = IssueComment.objects.get(id=comment_id)
    gitlab_issue_sync = GitlabIssueSync.objects.filter(issue=comment.issue).first()
    if gitlab_issue_sync:
        repo_sync = gitlab_issue_sync.repository_sync
        token = repo_sync.credentials.get("token")
        host = repo_sync.credentials.get("host", "https://gitlab.com").rstrip("/")
        if token:
            project_id = repo_sync.repository.repository_id
            issue_iid = gitlab_issue_sync.repo_issue_id

            gitlab_comment_sync = GitlabCommentSync.objects.filter(comment=comment).first()
            if gitlab_comment_sync:
                # Update
                requests.put(
                    f"{host}/api/v4/projects/{project_id}/issues/{issue_iid}/notes/{gitlab_comment_sync.repo_comment_id}",
                    headers={"PRIVATE-TOKEN": token},
                    json={"body": comment.comment_html}
                )
            else:
                # Create
                response = requests.post(
                    f"{host}/api/v4/projects/{project_id}/issues/{issue_iid}/notes",
                    headers={"PRIVATE-TOKEN": token},
                    json={"body": comment.comment_html}
                )
                if response.status_code == 201:
                    data = response.json()
                    GitlabCommentSync.objects.create(
                        comment=comment,
                        issue_sync=gitlab_issue_sync,
                        repo_comment_id=data["id"],
                        project=comment.project,
                        workspace=comment.workspace
                    )
