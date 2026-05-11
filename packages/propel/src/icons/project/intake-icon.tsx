/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export function IntakeIcon({ color = "currentColor", ...rest }: ISvgIcons) {
  const clipPathId = React.useId();

  return (
    <IconWrapper color={color} clipPathId={clipPathId} {...rest}>
      <path
        d="M7.37549 4.31C7.37549 3.35 7.36 3.1 7.12 2.77C7.07287 2.7 6.92 2.56 6.69 2.42C6.46459 2.28 6.27 2.21 6.19 2.2C5.92379 2.16 5.79 2.18 5.7 2.2C5.57897 2.23 5.46 2.29 5.21 2.42C3.2767 3.43 1.96 5.45 1.96 7.78C1.95856 11.12 4.66 13.82 8 13.82C11.337 13.82 14.04 11.12 14.04 7.78C14.0415 5.55 12.83 3.59 11.02 2.55C10.7222 2.37 10.62 1.99 10.79 1.69C10.9653 1.39 11.35 1.29 11.65 1.47C13.8238 2.73 15.29 5.08 15.29 7.78C15.2914 11.81 12.03 15.07 8 15.07C3.97345 15.07 0.71 11.81 0.71 7.78C0.708496 4.97 2.3 2.53 4.63 1.31C4.85634 1.2 5.1 1.06 5.38 0.99C5.6802 0.91 5.98 0.91 6.35 0.96C6.68627 1 7.05 1.17 7.34 1.35C7.62835 1.52 7.95 1.77 8.14 2.04C8.6434 2.75 8.63 3.43 8.63 4.31V8.93876L10.2251 7.34C10.4692 7.1 10.86 7.1 11.11 7.34C11.3529 7.58 11.35 7.98 11.11 8.22L8.44189 10.89C8.32477 11.01 8.17 11.07 8 11.07C7.83473 11.07 7.68 11.01 7.56 10.89L4.89111 8.22C4.64748 7.98 4.65 7.58 4.89 7.34C5.13519 7.1 5.53 7.1 5.78 7.34L7.37549 8.94V4.31474Z"
        fill={color}
      />
    </IconWrapper>
  );
}
