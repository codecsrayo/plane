/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { API_BASE_URL } from "@plane/constants";
import type { TTimezones } from "@plane/types";
// helpers
// api services
import { APIService, ApiError } from "@/services/api.service";

export class TimezoneService extends APIService {
  constructor() {
    super(API_BASE_URL);
  }

  async fetch(): Promise<TTimezones> {
    return this.get(`/api/timezones/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }
}

const timezoneService = new TimezoneService();

export default timezoneService;
