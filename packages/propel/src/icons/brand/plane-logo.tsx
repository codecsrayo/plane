/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";

import type { ISvgIcons } from "../type";

export function PlaneLogo({ width = "85", height = "52", className, color = "currentColor" }: ISvgIcons) {
  return (
    <svg
      width={width}
      height={height}
      viewBox="0 0 85 52"
      fill={color}
      xmlns="http://www.w3.org/2000/svg"
      className={className}
    >
      <path
        d="M44.3223 2.93C44.3223 0.75 46.61 -0.66 48.55 0.31L80.4551 16.27C82.9294 17.5 84.49 20.03 84.49 22.8V48.2487C84.4922 50.42 82.21 51.83 80.26 50.86L62.3281 41.89V22.7975C62.3281 20.03 60.77 17.5 58.29 16.26L44.3223 9.28V2.9264ZM0 2.93C8.01645e-05 0.75 2.29 -0.66 4.23 0.31L22.1582 9.28V28.3766C22.1582 31.14 23.72 33.67 26.2 34.91L40.1699 41.9V48.2487C40.1697 50.42 37.88 51.83 35.94 50.86L4.03711 34.91C1.56305 33.67 0 31.14 0 28.38V2.92543ZM22.1582 2.93C22.1583 0.75 24.44 -0.66 26.39 0.31L44.3223 9.28V28.3776C44.3223 31.14 45.89 33.67 48.36 34.91L62.3281 41.89V48.2487C62.3279 50.42 60.04 51.83 58.1 50.86L40.1699 41.9V22.7975C40.1699 20.03 38.61 17.5 36.13 16.26L22.1582 9.28V2.92543Z"
        fill={color}
      />
    </svg>
  );
}
