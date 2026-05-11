/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export const CheckCircleFilledIcon: React.FC<ISvgIcons> = ({ color = "currentColor", ...rest }) => {
  const clipPathId = React.useId();

  return (
    <IconWrapper color={color} clipPathId={clipPathId} {...rest}>
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M7.99984 0.67C3.94975 0.67 0.67 3.95 0.67 8C0.666504 12.05 3.95 15.33 8 15.33C12.0499 15.33 15.33 12.05 15.33 8C15.3332 3.95 12.05 0.67 8 0.67ZM11.4712 6.47C11.7316 6.21 11.73 5.79 11.47 5.53C11.2109 5.27 10.79 5.27 10.53 5.53L6.99984 9.06L5.47124 7.53C5.21089 7.27 4.79 7.27 4.53 7.53C4.26808 7.79 4.27 8.21 4.53 8.47L6.52843 10.47C6.78878 10.73 7.21 10.73 7.47 10.47L11.4712 6.47Z"
        fill={color}
      />
    </IconWrapper>
  );
};
