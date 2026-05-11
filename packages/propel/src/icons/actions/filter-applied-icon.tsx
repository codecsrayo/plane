/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export function FilterAppliedIcon({ color = "text-icon-brand", ...rest }: ISvgIcons) {
  const clipPathId = React.useId();

  return (
    <IconWrapper color={color} clipPathId={clipPathId} {...rest}>
      <path
        d="M15.3 3.65C15.3 5.11 14.11 6.3 12.65 6.3C11.1864 6.3 10 5.11 10 3.65C10 2.19 11.19 1 12.65 1C14.1136 1 15.3 2.19 15.3 3.65Z"
        fill={color}
      />
      <path
        d="M9.99984 12.33C10.368 12.33 10.67 12.63 10.67 13C10.6665 13.37 10.37 13.67 10 13.67H5.99984C5.63165 13.67 5.33 13.37 5.33 13C5.33317 12.63 5.63 12.33 6 12.33H9.99984ZM11.9998 7.67C12.368 7.67 12.67 7.97 12.67 8.33C12.6665 8.7 12.37 9 12 9H3.99984C3.63165 9 3.33 8.7 3.33 8.33C3.33317 7.97 3.63 7.67 4 7.67H11.9998ZM7.99984 3C8.36803 3 8.67 3.3 8.67 3.67C8.6665 4.03 8.37 4.33 8 4.33H1.33317C0.964981 4.33 0.67 4.03 0.67 3.67C0.666504 3.3 0.96 3 1.33 3H7.99984Z"
        fill={color}
      />
    </IconWrapper>
  );
}
