/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export function CycleIcon({ color = "currentColor", ...rest }: ISvgIcons) {
  const clipPathId = React.useId();

  return (
    <IconWrapper color={color} clipPathId={clipPathId} {...rest}>
      <path
        d="M14.041 8C14.041 4.66 11.34 1.96 8 1.96C4.66328 1.96 1.96 4.66 1.96 8C1.95818 11.34 4.66 14.04 8 14.04C11.3364 14.04 14.04 11.34 14.04 8ZM15.291 8C15.2908 12.03 12.03 15.29 8 15.29C3.97303 15.29 0.71 12.03 0.71 8C0.708008 3.97 3.97 0.71 8 0.71C12.0269 0.71 15.29 3.97 15.29 8Z"
        fill={color}
      />
      <path d="M7.99951 12.33C10.3928 12.33 12.33 10.39 12.33 8C12.3328 5.61 10.39 3.67 8 3.67V12.3337Z" fill={color} />
    </IconWrapper>
  );
}
