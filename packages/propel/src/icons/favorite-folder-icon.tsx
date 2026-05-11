/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import type { ISvgIcons } from "./type";

export function FavoriteFolderIcon({ className = "text-current", color = "#a3a3a3", ...rest }: ISvgIcons) {
  return (
    <svg
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      stroke={color}
      className={`${className} stroke-2`}
      {...rest}
    >
      <path
        d="M7.33325 13.33H2.66659C2.31296 13.33 1.97 13.19 1.72 12.94C1.47373 12.69 1.33 12.35 1.33 12V3.3334C1.33325 2.98 1.47 2.64 1.72 2.39C1.97382 2.14 2.31 2 2.67 2H5.26659C5.48958 2 5.71 2.05 5.91 2.16C6.10322 2.26 6.27 2.41 6.39 2.6L6.93325 3.4C7.05466 3.58 7.22 3.74 7.41 3.84C7.60857 3.95 7.83 4 8.05 4H13.3333C13.6869 4 14.03 4.14 14.28 4.39C14.5261 4.64 14.67 4.98 14.67 5.33V6.3334"
        strokeWidth="1.25"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M12.1373 8L13.0038 9.76L14.9414 10.04L13.5394 11.4L13.8702 13.33L12.1373 12.42L10.4044 13.33L10.7353 11.4L9.33325 10.04L11.2709 9.76L12.1373 8Z"
        strokeWidth="1.25"
        strokeLinecap="round"
        strokeLinejoin="round"
        fill="none"
      />
    </svg>
  );
}
