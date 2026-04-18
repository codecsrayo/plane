/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import type { TIssue } from "@plane/types";

/**
 * Payload describing which property changed and what the new value is.
 * Used for activity logging and analytics when a spreadsheet cell is edited.
 * `change_details` is intentionally `unknown` because the shape varies per column
 * (string id for state/priority, Date for dates, string[] for assignees/labels, etc.).
 * Callers narrow based on `changed_property`.
 */
export type TSpreadsheetColumnUpdatePayload = {
  changed_property: string;
  change_details: unknown;
};

/**
 * Shared signature for the `onChange` prop used across every spreadsheet column.
 * Previously each column declared `updates: any` locally — this type centralizes
 * the contract so the activity payload shape is consistent everywhere.
 */
export type TSpreadsheetColumnOnChange = (
  issue: TIssue,
  data: Partial<TIssue>,
  updates: TSpreadsheetColumnUpdatePayload
) => void;
