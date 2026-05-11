/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export function DropdownPropertyIcon({ color = "currentColor", ...rest }: ISvgIcons) {
  const clipPathId = React.useId();

  return (
    <IconWrapper color={color} clipPathId={clipPathId} {...rest}>
      <path
        d="M14.041 8C14.041 4.66 11.34 1.96 8 1.96C4.66328 1.96 1.96 4.66 1.96 8C1.95818 11.34 4.66 14.04 8 14.04C11.3364 14.04 14.04 11.34 14.04 8ZM10.2246 6.56C10.4687 6.31 10.86 6.31 11.11 6.56C11.3524 6.8 11.35 7.2 11.11 7.44L8.44141 10.11C8.32429 10.23 8.17 10.29 8 10.29C7.83424 10.29 7.67 10.23 7.56 10.11L4.89062 7.44C4.64693 7.2 4.65 6.8 4.89 6.56C5.1347 6.31 5.53 6.31 5.78 6.56L7.99902 8.78L10.2246 6.56ZM15.291 8C15.2908 12.03 12.03 15.29 8 15.29C3.97303 15.29 0.71 12.03 0.71 8C0.708008 3.97 3.97 0.71 8 0.71C12.0269 0.71 15.29 3.97 15.29 8Z"
        fill={color}
      />
    </IconWrapper>
  );
}
