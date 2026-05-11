/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export const GlobeIcon: React.FC<ISvgIcons> = ({ color = "currentColor", ...rest }) => {
  const clipPathId = React.useId();
  return (
    <IconWrapper color={color} clipPathId={clipPathId} {...rest}>
      <path
        d="M8.00195 0.71C12.0288 0.71 15.29 3.97 15.29 8C15.2928 12.03 12.03 15.29 8 15.29C3.97498 15.29 0.71 12.03 0.71 8C0.709961 3.97 3.97 0.71 8 0.71ZM1.99219 8.63C2.25614 11.19 4.13 13.28 6.58 13.87C5.52513 12.32 4.89 10.51 4.74 8.63H1.99219ZM11.2607 8.63C11.1145 10.51 10.48 12.32 9.42 13.87C11.8765 13.28 13.75 11.19 14.01 8.63H11.2607ZM5.99609 8.63C6.15854 10.48 6.86 12.24 8 13.7C9.14679 12.24 9.84 10.48 10.01 8.63H5.99609ZM9.42188 2.13C10.4789 3.68 11.11 5.49 11.26 7.38H14.0107C13.747 4.81 11.88 2.72 9.42 2.13ZM8.00098 2.3C6.85524 3.76 6.16 5.53 6 7.38H10.0068C9.84448 5.53 9.15 3.77 8 2.3ZM6.58105 2.13C4.12629 2.72 2.26 4.81 1.99 7.38H4.74219C4.88837 5.49 5.52 3.68 6.58 2.13Z"
        fill={color}
      />
    </IconWrapper>
  );
};
