/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export function PlaneNewIcon({ color = "currentColor", ...rest }: ISvgIcons) {
  return (
    <IconWrapper color={color} {...rest}>
      <path
        d="M10.3617 10.36V12.8365C10.3617 13.83 9.56 14.63 8.57 14.63H3.17365C2.18298 14.63 1.38 13.83 1.38 12.84V7.44221C1.37988 6.45 2.18 5.65 3.17 5.65H5.64726V8.56915C5.64726 9.56 6.45 10.36 7.44 10.36H10.3617Z"
        fill={color}
      />
      <path
        d="M14.6291 3.17V8.56797C14.6291 9.56 13.83 10.36 12.84 10.36H10.3625V7.44103C10.3625 6.45 9.56 5.65 8.57 5.65H5.64803V3.17365C5.64803 2.18 6.45 1.38 7.44 1.38H12.8361C13.8275 1.38 14.63 2.18 14.63 3.17Z"
        fill={color}
      />
    </IconWrapper>
  );
}
