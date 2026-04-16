/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { Tooltip2 } from "@blueprintjs/popover2";
import React, { useEffect, useRef, useState } from "react";
// helpers
import { cn } from "../utils";

export type TPosition =
  | "top"
  | "right"
  | "bottom"
  | "left"
  | "auto"
  | "auto-end"
  | "auto-start"
  | "bottom-left"
  | "bottom-right"
  | "left-bottom"
  | "left-top"
  | "right-bottom"
  | "right-top"
  | "top-left"
  | "top-right";

interface ITooltipProps {
  tooltipHeading?: string;
  tooltipContent: string | React.ReactNode;
  position?: TPosition;
  children: React.ReactElement;
  disabled?: boolean;
  className?: string;
  openDelay?: number;
  closeDelay?: number;
  isMobile?: boolean;
  renderByDefault?: boolean;
}

// Nota: el `= {} as ITooltipProps` final es defensivo. Con destructuring inline en el parámetro
// (`function Foo({ a = 1 }: Props) { ... }`) el bind revienta con "Cannot destructure property …
// of 'undefined' as it is undefined" si el primer argumento llega como `undefined` — los defaults
// por propiedad no cubren el caso del objeto entero. React garantiza un objeto de props vía JSX,
// pero HMR swaps, patrones render-as-value, HOCs con props mal tipados y boundaries de Fast
// Refresh pueden filtrar `undefined`. El cast a ITooltipProps es necesario porque `tooltipContent`
// y `children` están tipados como requeridos.
//
// `children` también se guardea explícitamente más abajo: el render path llama a
// `React.cloneElement(children, …)`, que revienta si `children` es `undefined`. Con el default
// `= {}` el destructuring ya no crashea, pero sin el guard de children el componente seguiría
// reventando una línea más abajo. Mismo patrón (default + guard temprano) que se aplicó en
// AppSidebarItem (commit 099091c) y en el Tooltip de propel (commit 00c84b0).
export function Tooltip({
  tooltipHeading,
  tooltipContent,
  position = "top",
  children,
  disabled = false,
  className = "",
  openDelay = 200,
  closeDelay,
  isMobile = false,

  //FIXME: tooltip should always render on hover and not by default, this is a temporary fix
  renderByDefault = true,
}: ITooltipProps = {} as ITooltipProps) {
  const toolTipRef = useRef<HTMLDivElement | null>(null);

  const [shouldRender, setShouldRender] = useState(renderByDefault);

  const onHover = () => {
    setShouldRender(true);
  };

  useEffect(() => {
    const element = toolTipRef.current as any;

    if (!element) return;

    element.addEventListener("mouseenter", onHover);

    return () => {
      element?.removeEventListener("mouseenter", onHover);
    };
  }, [toolTipRef, shouldRender]);

  // Guard explícito: `React.cloneElement(children, …)` en el renderTarget de Tooltip2 revienta
  // si `children` es `undefined`. Si no hay nada que anclar, no hay tooltip posible — salimos
  // temprano sin romper el subtree.
  if (!children) return null;

  if (!shouldRender) {
    return (
      <div ref={toolTipRef} className="flex h-full items-center">
        {children}
      </div>
    );
  }

  return (
    <Tooltip2
      disabled={disabled}
      hoverOpenDelay={openDelay}
      hoverCloseDelay={closeDelay}
      content={
        <div
          className={cn(
            "shadow-md relative z-50 block max-w-xs gap-1 overflow-hidden rounded-md bg-surface-1 p-2 text-11 break-words text-secondary",
            {
              hidden: isMobile,
            },
            className
          )}
        >
          {tooltipHeading && <h5 className="font-medium text-primary">{tooltipHeading}</h5>}
          {tooltipContent}
        </div>
      }
      position={position}
      renderTarget={({ isOpen: isTooltipOpen, ref: eleReference, ...tooltipProps }) =>
        React.cloneElement(children, {
          ref: eleReference,
          ...tooltipProps,
          ...children.props,
        })
      }
    />
  );
}
