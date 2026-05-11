/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import type { ISvgIcons } from "./type";

export function CalendarAfterIcon({ className = "fill-current", ...rest }: ISvgIcons) {
  return (
    <svg viewBox="0 0 24 24" className={`${className} `} fill="none" xmlns="http://www.w3.org/2000/svg" {...rest}>
      <g clipPath="url(#clip0_3309_70901)">
        <path
          d="M10.6125 17V15.875H14.625V7.8125H3.375V11.9375H2.25V4.25C2.25 3.95 2.36 3.69 2.59 3.46C2.8125 3.24 3.08 3.12 3.38 3.12H4.59375V2H5.8125V3.125H12.1875V2H13.4063V3.125H14.625C14.925 3.12 15.19 3.24 15.41 3.46C15.6375 3.69 15.75 3.95 15.75 4.25V15.875C15.75 16.18 15.64 16.44 15.41 16.66C15.1875 16.89 14.93 17 14.62 17H10.6125ZM6 18.24L5.2125 17.45L7.33125 15.31H0.9375V14.1875H7.33125L5.2125 12.05L6 11.26L9.4875 14.75L6 18.24ZM3.375 6.69H14.625V4.25H3.375V6.6875Z"
          fill="var(--text-color-secondary)"
        />
      </g>
      <defs>
        <clipPath id="clip0_3309_70901">
          <rect width="18" height="18" fill="white" transform="translate(0 0.5)" />
        </clipPath>
      </defs>
    </svg>
  );
}
