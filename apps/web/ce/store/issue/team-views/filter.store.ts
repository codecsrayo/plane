/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import type { IProjectViewIssuesFilter } from "@/store/issue/project-views";
import { ProjectViewIssuesFilter } from "@/store/issue/project-views";

/**
 * CE fallback for team-view issue filters. Real implementation lives in
 * `plane-web/store/issue/team-views`; this is what resolves in CE builds.
 */
export type ITeamViewIssuesFilter = IProjectViewIssuesFilter;

export class TeamViewIssuesFilter extends ProjectViewIssuesFilter implements ITeamViewIssuesFilter {}
