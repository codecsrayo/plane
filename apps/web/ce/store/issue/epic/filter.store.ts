/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import type { IProjectIssuesFilter } from "@/store/issue/project";
import { ProjectIssuesFilter } from "@/store/issue/project";

/**
 * CE fallback for epic filters. Real implementation lives in
 * `plane-web/store/issue/epic`; in CE builds the module resolver points
 * `@/plane-web/store/issue/epic` at this file, so the class IS instantiated
 * at runtime even though the previous audit note said "this class will
 * never be used". See `core/store/issue/root.store.ts::projectEpicsFilter`.
 *
 * The parent constructor already assigns `rootIssueStore`, so no body is
 * needed here — the `extends` is sufficient to satisfy the interface.
 */
export type IProjectEpicsFilter = IProjectIssuesFilter;

export class ProjectEpicsFilter extends ProjectIssuesFilter implements IProjectEpicsFilter {}
