/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { IconWrapper } from "../icon-wrapper";
import type { ISvgIcons } from "../type";

export function UpgradeIcon({ color = "currentColor", ...rest }: ISvgIcons) {
  return (
    <IconWrapper color={color} {...rest}>
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M8 1C4.13401 1 1 4.13 1 8C1 11.87 4.13 15 8 15C11.866 15 15 11.87 15 8C15 4.13 11.87 1 8 1ZM5.00457 7.55L7.55003 5C7.79853 4.76 8.2 4.76 8.45 5L10.9954 7.55C11.2439 7.8 11.24 8.2 11 8.45C10.7469 8.7 10.34 8.7 10.1 8.45L8.63636 6.99V10.5455C8.63636 10.9 8.35 11.18 8 11.18C7.64854 11.18 7.36 10.9 7.36 10.55V6.99085L5.90452 8.45C5.65601 8.7 5.25 8.7 5 8.45C4.75605 8.2 4.76 7.8 5 7.55Z"
        fill={color}
      />
    </IconWrapper>
  );
}
