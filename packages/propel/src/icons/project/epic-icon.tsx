/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export function EpicIcon({ color = "currentColor", ...rest }: ISvgIcons) {
  return (
    <IconWrapper color={color} {...rest}>
      <path
        d="M6.47557 3.58C6.25869 3.59 6.06 3.71 5.96 3.89L1.45506 11.53C1.34621 11.71 1.35 11.94 1.46 12.12C1.57303 12.3 1.78 12.42 2 12.42H14C14.2176 12.42 14.42 12.31 14.53 12.13C14.6467 11.95 14.66 11.73 14.56 11.55L11.5566 5.95C11.4711 5.79 11.31 5.67 11.13 5.64C10.9474 5.6 10.76 5.64 10.61 5.75L9.14159 6.87L7.01952 3.85C6.8987 3.68 6.69 3.58 6.48 3.58ZM8.48045 8.08C8.5777 8.22 8.73 8.31 8.9 8.34C9.07718 8.36 9.25 8.32 9.39 8.21L10.793 7.14L12.9785 11.22H3.06932L6.54686 5.32L8.48045 8.08Z"
        fill={color}
      />
    </IconWrapper>
  );
}
