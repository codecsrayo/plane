/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import type { IProjectIssuesFilter } from "@/store/issue/project";
import { ProjectIssuesFilter } from "@/store/issue/project";

/**
 * CE fallback for team-project work-item filters. Real implementation lives
 * in `plane-web/store/issue/team-project`; this is what resolves in CE builds.
 */
export type ITeamProjectWorkItemsFilter = IProjectIssuesFilter;

export class TeamProjectWorkItemsFilter
  extends ProjectIssuesFilter
  implements ITeamProjectWorkItemsFilter {}
