/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import type { IProjectIssuesFilter } from "@/store/issue/project";
import { ProjectIssuesFilter } from "@/store/issue/project";

/**
 * CE fallback for team issue filters. Real implementation lives in
 * `plane-web/store/issue/team`; this is what resolves in CE builds.
 */
export type ITeamIssuesFilter = IProjectIssuesFilter;

export class TeamIssuesFilter extends ProjectIssuesFilter implements ITeamIssuesFilter {}
