/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import type { IProjectViewIssues } from "@/store/issue/project-views";
import { ProjectViewIssues } from "@/store/issue/project-views";

/**
 * CE fallback for team-view issues. Real implementation lives in
 * `plane-web/store/issue/team-views`; this is what resolves in CE builds.
 */
export type ITeamViewIssues = IProjectViewIssues;

export class TeamViewIssues extends ProjectViewIssues implements ITeamViewIssues {}
