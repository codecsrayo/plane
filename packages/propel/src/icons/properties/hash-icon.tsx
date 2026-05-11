/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export function HashPropertyIcon({ color = "currentColor", ...rest }: ISvgIcons) {
  return (
    <IconWrapper color={color} {...rest}>
      <path
        d="M12.1025 1.38C12.443 1.44 12.67 1.76 12.62 2.1L12.1816 4.71H14C14.3451 4.71 14.62 4.99 14.62 5.33C14.625 5.68 14.35 5.96 14 5.96H11.9736L11.293 10.04H13.333C13.6782 10.04 13.96 10.32 13.96 10.67C13.9578 11.01 13.68 11.29 13.33 11.29H11.085L10.6162 14.1C10.5595 14.44 10.24 14.67 9.9 14.62C9.55703 14.56 9.33 14.24 9.38 13.9L9.81836 11.29H5.75195L5.2832 14.1C5.22646 14.44 4.9 14.67 4.56 14.62C4.22334 14.56 3.99 14.24 4.05 13.9L4.48438 11.29H2C1.65493 11.29 1.38 11.01 1.38 10.67C1.375 10.32 1.65 10.04 2 10.04H4.69238L5.37305 5.96H2.66699C2.32181 5.96 2.04 5.68 2.04 5.33C2.04217 4.99 2.32 4.71 2.67 4.71H5.58105L6.0498 1.9C6.10655 1.56 6.43 1.33 6.77 1.38C7.10985 1.44 7.34 1.76 7.28 2.1L6.84863 4.71H10.915L11.3838 1.9C11.4405 1.56 11.76 1.33 12.1 1.38ZM5.95996 10.04H10.0264L10.707 5.96H6.64062L5.95996 10.04Z"
        fill={color}
      />
    </IconWrapper>
  );
}
