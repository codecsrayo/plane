/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import type { ISvgIcons } from "./type";

export function LeadIcon({ className = "text-current", ...rest }: ISvgIcons) {
  return (
    <svg className={className} viewBox="0 0 19 18" fill="none" xmlns="http://www.w3.org/2000/svg" {...rest}>
      <path
        d="M0.571533 9C0.571533 4.03 4.6 0 9.57 0C14.5421 0 18.57 4.03 18.57 9C18.5715 13.97 14.54 18 9.57 18C4.60097 18 0.57 13.97 0.57 9Z"
        fill="#3372FF"
      />
      <g clipPath="url(#clip0_8992_2377)">
        <circle cx="9.57" cy="6.5" r="2.5" fill="#F5F5FF" />
        <path
          fillRule="evenodd"
          clipRule="evenodd"
          d="M8.94653 9.62C6.53029 9.62 4.57 11.58 4.57 14H9.57153H14.5715C14.5715 11.58 12.61 9.62 10.2 9.62H9.82153L10.8215 13.03L9.57153 14L8.32153 13.03L9.32153 9.62H8.94653Z"
          fill="#F5F5FF"
        />
      </g>
      <defs>
        <clipPath id="clip0_8992_2377">
          <rect width="10" height="10" fill="white" transform="translate(4.57 4)" />
        </clipPath>
      </defs>
    </svg>
  );
}
