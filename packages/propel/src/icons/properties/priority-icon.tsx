/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export function PriorityPropertyIcon({ color = "currentColor", ...rest }: ISvgIcons) {
  return (
    <IconWrapper color={color} {...rest}>
      <path
        d="M3.375 13.33V10.667C3.375 10.32 3.65 10.04 4 10.04C4.34518 10.04 4.62 10.32 4.62 10.67V13.334C4.62482 13.68 4.35 13.96 4 13.96C3.65493 13.96 3.38 13.68 3.38 13.33ZM7.375 13.33V6.66699C7.375 6.32 7.65 6.04 8 6.04C8.34518 6.04 8.62 6.32 8.62 6.67V13.334C8.62482 13.68 8.35 13.96 8 13.96C7.65493 13.96 7.38 13.68 7.38 13.33ZM11.375 13.33V2.66699C11.375 2.32 11.65 2.04 12 2.04C12.3452 2.04 12.62 2.32 12.62 2.67V13.334C12.6248 13.68 12.35 13.96 12 13.96C11.6549 13.96 11.38 13.68 11.38 13.33Z"
        fill={color}
      />
    </IconWrapper>
  );
}
