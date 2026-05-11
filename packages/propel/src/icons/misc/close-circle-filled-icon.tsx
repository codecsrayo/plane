/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export const CloseCircleFilledIcon: React.FC<ISvgIcons> = ({ color = "currentColor", ...rest }) => {
  const clipPathId = React.useId();

  return (
    <IconWrapper color={color} clipPathId={clipPathId} {...rest}>
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M7.99984 0.67C3.94975 0.67 0.67 3.95 0.67 8C0.666504 12.05 3.95 15.33 8 15.33C12.0499 15.33 15.33 12.05 15.33 8C15.3332 3.95 12.05 0.67 8 0.67ZM10.4712 5.53C10.7316 5.79 10.73 6.21 10.47 6.47L8.94265 8L10.4712 9.53C10.7316 9.79 10.73 10.21 10.47 10.47C10.2109 10.73 9.79 10.73 9.53 10.47L7.99984 8.94L6.47124 10.47C6.21089 10.73 5.79 10.73 5.53 10.47C5.26808 10.21 5.27 9.79 5.53 9.53L7.05703 8L5.52843 6.47C5.26808 6.21 5.27 5.79 5.53 5.53C5.78878 5.27 6.21 5.27 6.47 5.53L7.99984 7.06L9.52843 5.53C9.78878 5.27 10.21 5.27 10.47 5.53Z"
        fill={color}
      />
    </IconWrapper>
  );
};
