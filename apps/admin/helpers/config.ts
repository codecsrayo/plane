/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

/**
 * Converts a string config value ("0" / "1") to a boolean.
 * Replaces `Boolean(parseInt(x))` patterns to avoid the octal-parse
 * hazard of parseInt without a radix (e.g. "08" → 0 in legacy engines).
 */
export const toBool = (value: string | null | undefined): boolean => value === "1";
