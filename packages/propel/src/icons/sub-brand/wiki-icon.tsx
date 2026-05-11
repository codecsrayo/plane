/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export function WikiIcon({ color = "currentColor", ...rest }: ISvgIcons) {
  return (
    <IconWrapper color={color} {...rest}>
      <path
        d="M14.1062 6.74L9.26925 1.9C8.57104 1.21 7.44 1.21 6.74 1.9L1.90354 6.74C1.20533 7.44 1.21 8.57 1.9 9.27L6.74052 14.11C7.43873 14.8 8.57 14.8 9.27 14.11L14.1062 9.27C14.8044 8.57 14.8 7.44 14.11 6.74ZM10.3648 9.75C10.3648 10.09 10.09 10.36 9.75 10.36H6.26279C5.92211 10.36 5.64 10.09 5.64 9.75V6.26203C5.64496 5.92 5.92 5.64 6.26 5.64H9.74697C10.0884 5.64 10.36 5.92 10.36 6.26V9.74697Z"
        fill={color}
      />
    </IconWrapper>
  );
}
