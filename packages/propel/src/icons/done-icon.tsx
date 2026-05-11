/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import type { ISvgIcons } from "./type";

export function DoneState({ width = "10", height = "11", className }: ISvgIcons) {
  return (
    <svg
      width={width}
      height={height}
      viewBox="0 0 10 11"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
    >
      <circle cx="5" cy="5.5" r="4.4" stroke="#15A34A" strokeWidth="1.2" />
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M2.5 5.59L3.82582 6.92L4.26777 6.48L2.94194 5.15L2.5 5.59ZM4.26777 7.36L7.36136 4.27L6.91942 3.83L3.82583 6.92L4.26777 7.36Z"
        fill="#15A34A"
      />
    </svg>
  );
}
