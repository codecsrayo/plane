/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export function AddCircleIcon({ color = "currentColor", ...rest }: ISvgIcons) {
  const clipPathId = React.useId();

  return (
    <IconWrapper color={color} clipPathId={clipPathId} {...rest}>
      <path
        d="M14.0413 8C14.0413 4.66 11.34 1.96 8 1.96C4.66352 1.96 1.96 4.66 1.96 8C1.95843 11.34 4.66 14.04 8 14.04C11.3367 14.04 14.04 11.34 14.04 8ZM7.37524 10.67V8.625H5.33325C4.98818 8.62 4.71 8.35 4.71 8C4.70825 7.65 4.99 7.38 5.33 7.38H7.37524V5.33301C7.37524 4.99 7.66 4.71 8 4.71C8.34527 4.71 8.63 4.99 8.63 5.33V7.375H10.6663C11.0114 7.38 11.29 7.65 11.29 8C11.2911 8.35 11.01 8.62 10.67 8.62H8.62524V10.666C8.62524 11.01 8.35 11.29 8 11.29C7.65507 11.29 7.38 11.01 7.38 10.67ZM15.2913 8C15.2911 12.03 12.03 15.29 8 15.29C3.97328 15.29 0.71 12.03 0.71 8C0.708252 3.97 3.97 0.71 8 0.71C12.0272 0.71 15.29 3.97 15.29 8Z"
        fill={color}
      />
    </IconWrapper>
  );
}
