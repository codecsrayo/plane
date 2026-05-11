/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import type { ISvgIcons } from "./type";

export function BarIcon({ className = "", ...rest }: ISvgIcons) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg" {...rest}>
      <path
        d="M0 12.59C0 11.48 0.9 10.59 2 10.59H3.64706C4.75163 10.59 5.65 11.48 5.65 12.59V23.9977H0V12.5859Z"
        fill="currentColor"
      />
      <path
        d="M9.17773 2C9.17773 0.9 10.07 0 11.18 0H12.8248C13.9294 0 14.82 0.9 14.82 2V24H9.17773V2Z"
        fill="currentColor"
      />
      <path
        d="M18.3535 8.35C18.3535 7.25 19.25 6.35 20.35 6.35H22.0006C23.1051 6.35 24 7.25 24 8.35V23.9986H18.3535V8.35156Z"
        fill="currentColor"
      />
    </svg>
  );
}
