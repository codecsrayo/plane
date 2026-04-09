# Copyright (c) 2023-present Plane Software, Inc. and contributors
# SPDX-License-Identifier: AGPL-3.0-only
# See the LICENSE file for details.

"""
Background task: initial bulk import of existing GitHub issues when a
GithubRepositorySync is first created.

Only issues that haven't been imported yet (no GithubIssueSync record)
are created — the webhook handler covers all future events.
"""

import logging

import requests
from celery import shared_task

from plane.db.models import (
    GithubRepositorySync,
    GithubIssueSync,
    Issue,
    State,
)

logger = logging.getLogger(__name__)


@shared_task
def github_initial_issue_sync_task(repo_sync_id: str) -> None:
    """
    Pull all existing open/closed issues from a GitHub repository and create
    the corresponding Plane Issues + GithubIssueSync records.

    Args:
        repo_sync_id: UUID of the GithubRepositorySync to seed.
    """
    try:
        sync = (
            GithubRepositorySync.objects.select_related(
                "repository",
                "project",
                "workspace",
                "workspace_integration",
                "actor",
            )
            .get(pk=repo_sync_id)
        )
    except GithubRepositorySync.DoesNotExist:
        logger.warning("github_initial_issue_sync_task: sync %s not found", repo_sync_id)
        return

    # --- Obtain GitHub installation token -----------------------------------
    try:
        from plane.utils.github_app import get_installation_access_token

        installation_id = sync.workspace_integration.metadata.get("installation_id")
        if not installation_id:
            logger.warning(
                "github_initial_issue_sync_task: no installation_id for workspace_integration=%s",
                sync.workspace_integration_id,
            )
            return

        token = get_installation_access_token(installation_id)
        if not token:
            logger.warning(
                "github_initial_issue_sync_task: could not obtain token for installation=%s",
                installation_id,
            )
            return
    except Exception:
        logger.exception(
            "github_initial_issue_sync_task: error obtaining token for sync=%s", repo_sync_id
        )
        return

    owner = sync.repository.owner
    repo_name = sync.repository.name
    project = sync.project

    headers = {
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
    }

    # Resolve open/closed states from sync credentials
    credentials = sync.credentials or {}
    open_state_id = credentials.get("issue_open_state")
    closed_state_id = credentials.get("issue_closed_state")

    open_state = None
    closed_state = None

    if open_state_id:
        open_state = State.objects.filter(pk=open_state_id, project=project).first()
    if closed_state_id:
        closed_state = State.objects.filter(pk=closed_state_id, project=project).first()

    # Fallback: use the project's default state for open issues
    if not open_state:
        open_state = (
            State.objects.filter(project=project, default=True).first()
            or State.objects.filter(project=project).first()
        )

    # Build set of already-synced GitHub issue IDs to avoid duplicates
    existing_gh_ids = set(
        GithubIssueSync.objects.filter(repository_sync=sync).values_list(
            "github_issue_id", flat=True
        )
    )

    page = 1
    imported = 0
    skipped = 0

    while True:
        try:
            resp = requests.get(
                f"https://api.github.com/repos/{owner}/{repo_name}/issues",
                headers=headers,
                params={
                    "state": "all",
                    "per_page": 100,
                    "page": page,
                    "sort": "created",
                    "direction": "asc",
                },
                timeout=30,
            )
        except Exception:
            logger.exception(
                "github_initial_issue_sync_task: request error on page=%s for sync=%s",
                page,
                repo_sync_id,
            )
            break

        if resp.status_code == 401:
            logger.error(
                "github_initial_issue_sync_task: 401 Unauthorized — token may have expired for sync=%s",
                repo_sync_id,
            )
            break

        if resp.status_code != 200:
            logger.warning(
                "github_initial_issue_sync_task: GitHub API returned %s on page=%s for sync=%s",
                resp.status_code,
                page,
                repo_sync_id,
            )
            break

        gh_issues = resp.json()
        if not gh_issues:
            break  # No more pages

        for gh_issue in gh_issues:
            # GitHub's /issues endpoint returns PRs too — skip them
            if "pull_request" in gh_issue:
                skipped += 1
                continue

            gh_issue_id = gh_issue["id"]

            # Skip issues that are already tracked
            if gh_issue_id in existing_gh_ids:
                skipped += 1
                continue

            # Determine target state based on GitHub issue state
            gh_state = gh_issue.get("state", "open")
            if gh_state == "closed" and closed_state:
                target_state = closed_state
            else:
                target_state = open_state

            state_kwargs = {"state": target_state} if target_state else {}

            try:
                plane_issue = Issue.objects.create(
                    name=gh_issue["title"][:255],
                    description_html=gh_issue.get("body") or "",
                    project=project,
                    workspace=sync.workspace,
                    created_by=sync.actor,
                    updated_by=sync.actor,
                    **state_kwargs,
                )

                GithubIssueSync.objects.create(
                    issue=plane_issue,
                    repository_sync=sync,
                    repo_issue_id=gh_issue["number"],
                    github_issue_id=gh_issue_id,
                    issue_url=gh_issue.get("html_url", ""),
                    project=project,
                    workspace=sync.workspace,
                )

                existing_gh_ids.add(gh_issue_id)
                imported += 1

            except Exception:
                logger.exception(
                    "github_initial_issue_sync_task: failed to import gh_issue=%s for sync=%s",
                    gh_issue_id,
                    repo_sync_id,
                )

        page += 1

    logger.info(
        "github_initial_issue_sync_task: sync=%s done — imported=%s skipped=%s",
        repo_sync_id,
        imported,
        skipped,
    )
