/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import type { AnyExtension, Extensions } from "@tiptap/core";
import DocumentExtension from "@tiptap/extension-document";
import HeadingExtension from "@tiptap/extension-heading";
import TextExtension from "@tiptap/extension-text";

export const TitleExtensions: Extensions = [
  DocumentExtension.extend({
    content: "heading",
  }),
  HeadingExtension.configure({
    levels: [1],
  }) as AnyExtension,
  TextExtension,
];
