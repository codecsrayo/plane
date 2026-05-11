/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export function AudioFileIcon({ color = "currentColor", ...rest }: ISvgIcons) {
  return (
    <IconWrapper color={color} {...rest}>
      <path
        d="M7.33325 14V1.99992C7.33325 1.63 7.63 1.33 8 1.33C8.36811 1.33 8.67 1.63 8.67 2V13.9999C8.66659 14.37 8.37 14.67 8 14.67C7.63173 14.67 7.33 14.37 7.33 14ZM4.33325 12V3.99992C4.33325 3.63 4.63 3.33 5 3.33C5.36811 3.33 5.67 3.63 5.67 4V11.9999C5.66659 12.37 5.37 12.67 5 12.67C4.63173 12.67 4.33 12.37 4.33 12ZM10.3333 12V3.99992C10.3333 3.63 10.63 3.33 11 3.33C11.3681 3.33 11.67 3.63 11.67 4V11.9999C11.6666 12.37 11.37 12.67 11 12.67C10.6317 12.67 10.33 12.37 10.33 12ZM1.33325 9.33V6.66659C1.33325 6.3 1.63 6 2 6C2.36811 6 2.67 6.3 2.67 6.67V9.33325C2.66659 9.7 2.37 10 2 10C1.63173 10 1.33 9.7 1.33 9.33ZM13.3333 9.33V6.66659C13.3333 6.3 13.63 6 14 6C14.3681 6 14.67 6.3 14.67 6.67V9.33325C14.6666 9.7 14.37 10 14 10C13.6317 10 13.33 9.7 13.33 9.33Z"
        fill={color}
      />
    </IconWrapper>
  );
}
