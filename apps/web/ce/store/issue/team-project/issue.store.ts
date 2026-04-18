/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import type { IProjectIssues } from "@/store/issue/project";
import { ProjectIssues } from "@/store/issue/project";

/**
 * CE fallback for team-project work-items. Real implementation lives in
 * `plane-web/store/issue/team-project`; this is what resolves in CE builds.
 */
export type ITeamProjectWorkItems = IProjectIssues;

export class TeamProjectWorkItems extends ProjectIssues implements ITeamProjectWorkItems {}
