/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import TextAlignExtension from "@tiptap/extension-text-align";

export type TTextAlign = "left" | "center" | "right";

export const CustomTextAlignExtension = TextAlignExtension.configure({
  alignments: ["left", "center", "right"],
  types: ["heading", "paragraph"],
});
