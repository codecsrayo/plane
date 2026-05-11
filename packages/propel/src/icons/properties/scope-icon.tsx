/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export function ScopePropertyIcon({ color = "currentColor", ...rest }: ISvgIcons) {
  const clipPathId = React.useId();

  return (
    <IconWrapper color={color} clipPathId={clipPathId} {...rest}>
      <path
        d="M8.00049 0.71C12.0274 0.71 15.29 3.97 15.29 8C15.2913 12.03 12.03 15.29 8 15.29C3.97352 15.29 0.71 12.03 0.71 8C0.708496 3.97 3.97 0.71 8 0.71ZM7.37549 4V1.98926C4.53761 2.28 2.28 4.54 1.99 7.38H4.00049C4.34552 7.38 4.63 7.65 4.63 8C4.62531 8.34 4.35 8.62 4 8.62H1.99072C2.28252 11.46 4.54 13.72 7.38 14.01V12C7.37549 11.65 7.66 11.38 8 11.38C8.34552 11.38 8.63 11.65 8.63 12V14.0078C11.4627 13.72 13.72 11.46 14.01 8.62H12.0005C11.6554 8.62 11.38 8.35 11.38 8C11.3755 7.65 11.66 7.38 12 7.38H14.0093C13.7178 4.54 11.46 2.28 8.63 1.99V4C8.62531 4.34 8.35 4.62 8 4.62C7.65542 4.62 7.38 4.35 7.38 4Z"
        fill={color}
      />
    </IconWrapper>
  );
}
