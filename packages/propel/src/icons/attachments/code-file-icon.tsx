/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export function CodeFileIcon({ color = "currentColor", ...rest }: ISvgIcons) {
  return (
    <IconWrapper color={color} {...rest}>
      <path
        d="M9.4687 1.39C9.8056 1.46 10.02 1.8 9.94 2.14L7.27632 14.14C7.20136 14.47 6.87 14.69 6.53 14.61C6.19351 14.54 5.98 14.2 6.06 13.86L8.72261 1.86C8.79753 1.53 9.13 1.32 9.47 1.39ZM4.22456 4.22C4.46865 3.98 4.86 3.98 5.11 4.22C5.35237 4.47 5.35 4.86 5.11 5.11L2.21675 8L5.10835 10.89C5.35227 11.14 5.35 11.53 5.11 11.78C4.86433 12.02 4.47 12.02 4.22 11.78L0.890578 8.44C0.6465 8.2 0.65 7.8 0.89 7.56L4.22456 4.22ZM10.8906 4.22C11.1347 3.98 11.53 3.98 11.78 4.22L15.1084 7.56C15.3524 7.8 15.35 8.2 15.11 8.44L11.7753 11.78C11.5313 12.02 11.13 12.02 10.89 11.78C10.6468 11.53 10.65 11.14 10.89 10.89L13.7822 8L10.8906 5.11C10.6469 4.86 10.65 4.47 10.89 4.22Z"
        fill={color}
      />
    </IconWrapper>
  );
}
