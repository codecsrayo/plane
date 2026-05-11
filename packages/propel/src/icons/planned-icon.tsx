/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import type { ISvgIcons } from "./type";

export function PlannedState({ width = "10", height = "11", className }: ISvgIcons) {
  return (
    <svg
      width={width}
      height={height}
      viewBox="0 0 12 13"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
    >
      <g clipPath="url(#clip0_3180_28635)">
        <path
          fillRule="evenodd"
          clipRule="evenodd"
          d="M7.11853 4.7C7.20073 4.89 7.38 5.02 7.6 5.02C7.88848 5.02 8.13 4.78 8.13 4.48L8.13344 3.93C8.1348 3.75 8.05 3.58 7.91 3.48C7.76203 3.38 7.58 3.35 7.42 3.42L3.97959 4.78C3.77547 4.86 3.64 5.06 3.64 5.28L3.64077 9.09C3.64077 9.28 3.74 9.45 3.9 9.55C4.05347 9.65 4.25 9.66 4.41 9.57L4.90523 9.31C5.16402 9.17 5.26 8.84 5.13 8.57C5.04115 8.4 4.87 8.3 4.7 8.28L4.69795 5.66L7.11853 4.7Z"
          fill="#455068"
        />
        <path
          fillRule="evenodd"
          clipRule="evenodd"
          d="M5.00428 3.07C5.08648 3.26 5.27 3.39 5.48 3.39C5.77422 3.39 6.01 3.15 6.01 2.85L6.01918 2.3C6.02054 2.12 5.94 1.95 5.79 1.85C5.64777 1.75 5.46 1.72 5.3 1.79L1.86534 3.15C1.66121 3.23 1.53 3.43 1.53 3.65L1.52652 7.46C1.52652 7.65 1.62 7.82 1.78 7.92C1.93922 8.02 2.14 8.03 2.3 7.94L2.79097 7.68C3.04977 7.54 3.15 7.21 3.01 6.94C2.92689 6.77 2.76 6.66 2.58 6.65L2.5837 4.03L5.00428 3.07Z"
          fill="#455068"
        />
        <path
          fillRule="evenodd"
          clipRule="evenodd"
          d="M10.473 9.35C10.4728 9.57 10.34 9.77 10.13 9.85L6.70129 11.21C6.53874 11.28 6.36 11.26 6.21 11.15C6.06867 11.05 5.98 10.88 5.98 10.71L5.98288 6.9C5.98288 6.68 6.12 6.47 6.32 6.39L9.7572 5.04C9.91981 4.97 10.1 4.99 10.25 5.09C10.3899 5.2 10.48 5.36 10.48 5.54L10.473 9.35ZM9.41784 6.33L7.04006 7.27L7.04006 9.91L9.41605 8.97L9.41784 6.33Z"
          fill="#455068"
        />
      </g>
      <defs>
        <clipPath id="clip0_3180_28635">
          <rect width="12" height="12" fill="white" transform="translate(0 0.5)" />
        </clipPath>
      </defs>
    </svg>
  );
}
