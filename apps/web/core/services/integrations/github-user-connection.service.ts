/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 *
 * Service for the GitHub personal-account OAuth connection flow.
 *
 * Separated from the page component so it can be reused, mocked in tests,
 * and swapped independently of UI concerns.
 *
 * Backend contract:
 *   POST /api/auth/github/user-callback/  { code: string }
 *   → 200 { id, provider, metadata? }
 */

import { API_BASE_URL } from "@plane/constants";
import { ApiError, APIService } from "@/services/api.service";

export type TUserConnectionResponse = {
  id: string;
  provider: string;
  metadata?: Record<string, unknown> | null;
};

export class GithubUserConnectionService extends APIService {
  constructor() {
    super(API_BASE_URL);
  }

  async connectPersonalAccount(code: string): Promise<TUserConnectionResponse> {
    return this.post("/api/auth/github/user-callback/", { code })
      .then((res) => res?.data)
      .catch((err) => {
        throw new ApiError(err?.response);
      });
  }
}
