/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import type { IProjectIssues } from "@/store/issue/project";
import { ProjectIssues } from "@/store/issue/project";

/**
 * CE fallback for the epics issue store. Real implementation lives in
 * `plane-web/store/issue/epic`; this is what resolves in CE builds.
 */
export type IProjectEpics = IProjectIssues;

export class ProjectEpics extends ProjectIssues implements IProjectEpics {}
