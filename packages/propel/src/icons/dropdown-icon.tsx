/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import type { ISvgIcons } from "./type";

export function DropdownIcon({ className = "text-current", ...rest }: ISvgIcons) {
  return (
    <svg
      viewBox="0 0 7 5"
      className={`${className} stroke-2`}
      fill="currentColor"
      xmlns="http://www.w3.org/2000/svg"
      {...rest}
    >
      <path d="M2.77267 4.02L0.457719 1.79C0.162864 1.5 0.1 1.17 0.26 0.8C0.418861 0.43 0.71 0.25 1.13 0.25H5.72716C6.14685 0.25 6.44 0.43 6.6 0.8C6.76012 1.17 6.69 1.5 6.4 1.79L4.08357 4.02C3.98662 4.11 3.88 4.18 3.78 4.23C3.66918 4.27 3.55 4.29 3.43 4.29C3.30328 4.29 3.19 4.27 3.08 4.23C2.97191 4.18 2.87 4.11 2.77 4.02Z" />
    </svg>
  );
}
