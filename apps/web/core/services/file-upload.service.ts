/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import type { AxiosRequestConfig } from "axios";
import { CancelToken, isCancel } from "axios";
// services
import { APIService, ApiError } from "@/services/api.service";

export class FileUploadService extends APIService {
  private cancelSource: any;

  async uploadFile(
    url: string,
    data: FormData,
    uploadProgressHandler?: AxiosRequestConfig["onUploadProgress"]
  ): Promise<void> {
    this.cancelSource = CancelToken.source();
    return this.post(url, data, {
      headers: {
        "Content-Type": "multipart/form-data",
      },
      cancelToken: this.cancelSource.token,
      withCredentials: false,
      onUploadProgress: uploadProgressHandler,
    })
      .then((response) => response?.data)
      .catch((error) => {
        if (isCancel(error)) {
          console.error("File upload error:", error.message);
        } else {
          throw new ApiError(error?.response);
        }
      });
  }

  cancelUpload() {
    this.cancelSource.cancel("Upload canceled");
  }
}
