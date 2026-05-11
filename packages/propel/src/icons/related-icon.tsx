/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import type { ISvgIcons } from "./type";

export function RelatedIcon({ className = "text-current", ...rest }: ISvgIcons) {
  return (
    <svg
      viewBox="0 0 24 24"
      className={`${className} stroke-2`}
      stroke="currentColor"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      {...rest}
    >
      <path
        d="M20 13V6C20 5.47 19.79 4.96 19.41 4.59C19.0391 4.21 18.53 4 18 4H4C3.46957 4 2.96 4.21 2.59 4.59C2.21071 4.96 2 5.47 2 6V20C2 20.53 2.21 21.04 2.59 21.41C2.96086 21.79 3.47 22 4 22H11"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path d="M12.125 19.25L9 16.12L12.125 13" strokeLinecap="round" strokeLinejoin="round" />
      <path
        d="M20 22V18.1818C20 17.6 19.74 17.05 19.27 16.64C18.7989 16.23 18.16 16 17.5 16H10"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
