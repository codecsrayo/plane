/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import type { ISvgIcons } from "./type";

export function CalendarBeforeIcon({ className = "fill-current", ...rest }: ISvgIcons) {
  return (
    <svg viewBox="0 0 24 24" className={`${className} `} fill="none" xmlns="http://www.w3.org/2000/svg" {...rest}>
      <g clipPath="url(#clip0_3309_70907)">
        <path
          d="M10.6125 16.5V15.375H14.625V7.3125H3.375V11.4375H2.25V3.75C2.25 3.45 2.36 3.19 2.59 2.96C2.8125 2.74 3.08 2.62 3.38 2.62H4.59375V1.5H5.8125V2.625H12.1875V1.5H13.4062V2.625H14.625C14.925 2.62 15.19 2.74 15.41 2.96C15.6375 3.19 15.75 3.45 15.75 3.75V15.375C15.75 15.68 15.64 15.94 15.41 16.16C15.1875 16.39 14.93 16.5 14.62 16.5H10.6125ZM3.375 6.19H14.625V3.75H3.375V6.1875Z"
          fill="var(--text-color-secondary)"
        />
        <g clipPath="url(#clip1_3309_70907)">
          <path
            d="M3.99967 17.17L1.33301 14.5L3.99967 11.83L4.34967 12.18L2.28301 14.25H8V14.75H2.28301L4.34967 16.82L3.99967 17.17Z"
            fill="var(--text-color-secondary)"
            stroke="var(--text-color-secondary)"
            strokeWidth="0.5"
          />
        </g>
      </g>
      <defs>
        <clipPath id="clip0_3309_70907">
          <rect width="18" height="18" fill="white" />
        </clipPath>
        <clipPath id="clip1_3309_70907">
          <rect width="8" height="8" fill="white" transform="translate(0 10.5)" />
        </clipPath>
      </defs>
    </svg>
  );
}
