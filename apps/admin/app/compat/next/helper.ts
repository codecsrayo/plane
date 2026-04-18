/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

// lib
import { logger } from "@/lib/logger";

/**
 * Ensures that a URL has a trailing slash while preserving query parameters and fragments
 * @param url - The URL to process
 * @returns The URL with a trailing slash added to the pathname (if not already present)
 */
export function ensureTrailingSlash(url: string | null | undefined): string {
  // Defensive: although the compile-time type is 'string', this helper is on the
  // next/link compat path where callers occasionally pass an undefined 'href'
  // (e.g. 'router.push(maybe?.link)'). Returning '' keeps RRLink / navigate as
  // safe no-ops instead of a 'Cannot read properties of undefined' at runtime.
  if (!url) return "";
  try {
    const fallbackBaseUrl =
      typeof window !== "undefined" && window.location.origin ? window.location.origin : "http://dummy.com";
    // Handle relative URLs by creating a URL object with a fallback base URL
    const urlObj = new URL(url, fallbackBaseUrl);

    // Don't modify root path
    if (urlObj.pathname === "/") {
      return url;
    }

    // Add trailing slash if it doesn't exist
    if (!urlObj.pathname.endsWith("/")) {
      urlObj.pathname += "/";
    }

    // For relative URLs, return just the path + search + hash
    if (url.startsWith("/")) {
      return urlObj.pathname + urlObj.search + urlObj.hash;
    }

    // For absolute URLs, return the full URL
    return urlObj.toString();
  } catch (error) {
    // If URL parsing fails, return the original URL
    logger.warn("Failed to parse URL for trailing slash enforcement:", url, error);
    return url;
  }
}
