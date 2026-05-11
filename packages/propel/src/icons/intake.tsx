/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import type { ISvgIcons } from "./type";

export function Intake({ className = "text-current", ...rest }: ISvgIcons) {
  return (
    <svg
      viewBox="0 0 16 16"
      className={`${className}`}
      stroke="currentColor"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      strokeWidth="1.25"
      strokeLinecap="round"
      strokeLinejoin="round"
      {...rest}
    >
      <path d="M12.1599 3.6V9.60688L8.04358 12.08V6.04325L12.1599 3.6Z" />
      <path d="M5.98547 10.87V4.82938L10.1018 2.39" />
      <path d="M3.89087 9.61V3.57059L8.00723 1.13" />
      <path d="M1.06909 8.77V13.3887C1.06909 14.18 1.72 14.83 2.51 14.83H13.4909C14.2836 14.83 14.93 14.18 14.93 13.39V8.77051" />
    </svg>
  );
}
