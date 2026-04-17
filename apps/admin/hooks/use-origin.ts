/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useMemo } from "react";

/**
 * Returns the current window origin as a stable memoised value.
 *
 * Using a bare `typeof window !== "undefined" ? window.location.origin : ""`
 * expression directly in JSX (or as a module-level constant) can produce an
 * empty string on the first render pass before hydration completes, causing
 * `CopyField` to display — and let users copy — a partial URL such as
 * `/api/github/callback/` instead of `https://example.com/api/github/callback/`.
 *
 * Wrapping it in `useMemo` guarantees the value is evaluated at component
 * mount time (i.e. always in a fully hydrated browser context) and is never
 * recomputed on subsequent re-renders.
 */
export function useOrigin(): string {
  return useMemo(() => (typeof window !== "undefined" ? window.location.origin : ""), []);
}
