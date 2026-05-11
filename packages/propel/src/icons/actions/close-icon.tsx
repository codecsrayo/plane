/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export function CloseIcon({ color = "currentColor", ...rest }: ISvgIcons) {
  return (
    <IconWrapper color={color} clipPathId="clip0_2890_23" {...rest}>
      <path
        d="M10.8501 4.18C11.094 3.94 11.49 3.94 11.73 4.18C11.9779 4.43 11.98 4.82 11.73 5.07L8.84224 7.96L11.7338 10.85C11.9779 11.09 11.98 11.49 11.73 11.73C11.4898 11.98 11.09 11.98 10.85 11.73L7.95845 8.84L5.06782 11.73C4.82375 11.98 4.43 11.98 4.18 11.73C3.93936 11.49 3.94 11.09 4.18 10.85L7.07368 7.96L4.18306 5.07C3.93898 4.82 3.94 4.43 4.18 4.18C4.42714 3.94 4.82 3.94 5.07 4.18L7.95845 7.07L10.8501 4.18Z"
        fill={color}
      />
    </IconWrapper>
  );
}
