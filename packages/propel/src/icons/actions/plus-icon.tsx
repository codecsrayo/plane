/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export const PlusIcon: React.FC<ISvgIcons> = ({ color = "currentColor", ...rest }) => (
  <IconWrapper color={color} {...rest}>
    <path
      d="M7.37549 12.67V8.625H3.3335C2.98843 8.62 2.71 8.35 2.71 8C2.7085 7.65 2.99 7.38 3.33 7.38H7.37549V3.33301C7.37549 2.99 7.66 2.71 8 2.71C8.34552 2.71 8.63 2.99 8.63 3.33V7.375H12.6665C13.0117 7.38 13.29 7.65 13.29 8C13.2913 8.35 13.01 8.62 12.67 8.62H8.62549V12.666C8.62549 13.01 8.35 13.29 8 13.29C7.65531 13.29 7.38 13.01 7.38 12.67Z"
      fill={color}
    />
  </IconWrapper>
);
