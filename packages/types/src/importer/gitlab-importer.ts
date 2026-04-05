/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

export interface IGitlabServiceImportFormData {
  project_id: string;
  gitlab_project_id: string | number;
  gitlab_token: string;
  import_labels?: boolean;
  import_comments?: boolean;
}

export interface IGitlabRepoInfo {
  id: string;
  full_name: string;
  name: string;
  owner: string;
  description: string;
  private: boolean;
  url: string;
  issues_count: number;
}
