/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { API_BASE_URL } from "@plane/constants";
import { CanceledError, isCancel } from "axios";
// api service
import { APIService } from "../api.service";

/**
 * Service class for handling file upload operations
 * Handles file uploads
 * @extends {APIService}
 */
export class FileUploadService extends APIService {
  private abortController: AbortController | null = null;

  constructor(BASE_URL?: string) {
    super(BASE_URL || API_BASE_URL);
  }

  /**
   * Uploads a file to the specified signed URL
   * @param {string} url - The URL to upload the file to
   * @param {FormData} data - The form data to upload
   * @returns {Promise<void>} Promise resolving to void
   * @throws {Error} If the request fails
   */
  async uploadFile(url: string, data: FormData): Promise<void> {
    this.abortController = new AbortController();
    return this.post(url, data, {
      headers: {
        "Content-Type": "multipart/form-data",
      },
      signal: this.abortController.signal,
      withCredentials: false,
    })
      .then((response) => response?.data)
      .catch((error) => {
        if (isCancel(error) || error instanceof CanceledError) {
          console.log(error.message);
        } else {
          throw error?.response?.data;
        }
      });
  }

  /**
   * Cancels the upload
   */
  cancelUpload() {
    this.abortController?.abort("Upload canceled");
  }
}
