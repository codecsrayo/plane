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

// Defensa en dos capas contra `props` inválido:
//
// 1. Default del parámetro (`= {} as ITooltipProps`): atrapa el caso en que el componente es
//    invocado con `undefined` (render-as-value en libs headless, HMR/Fast Refresh swaps,
//    spreads de `undefined`, HOCs con props mal tipados). Los defaults por propiedad NO cubren
//    el caso del objeto entero — si `props` es `undefined`, la destructuración inline en el
//    parámetro revienta con "Cannot destructure property … of 'props' as it is undefined"
//    antes de que cualquier default por-prop llegue a evaluarse.
//
// 2. Nullish-coalesce en el cuerpo (`props ?? ({} as ITooltipProps)`): el default de parámetro
//    SOLO se dispara con `undefined`, no con `null`. Si un HOC o render prop mal tipado pasa
//    `null` explícito, volveríamos a crashear con "as it is null". El `??` absorbe ese caso.
//    Por eso la destructuración se hizo migrar al cuerpo — con inline-en-parámetro no hay
//    dónde meter el `??`.
//
// Guard explícito de `children` más abajo: `React.cloneElement(children, …)` en el renderTarget
// de Tooltip2 revienta si `children` es `undefined`. Si no hay nada que anclar, no hay tooltip
// posible — salimos temprano sin romper el subtree.
//
// El cast a `ITooltipProps` es necesario porque `tooltipContent` y `children` son requeridos
// en el tipo. Mismo patrón aplicado en AppSidebarItem (commit 099091c) y en el Tooltip de
// propel (commit 00c84b0).
export function Tooltip(props: ITooltipProps = {} as ITooltipProps) {
  const {
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
  } = props ?? ({} as ITooltipProps);
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

  // Normaliza `children` para que `cloneElement(…, { ref })` no dispare:
  //   "Warning: Function components cannot be given refs. … Check the render
  //    method of `TooltipTrigger`."
  // Si children es un function component sin forwardRef, no puede recibir
  // ref — lo envolvemos en un <span> transparente. Mismo helper que en
  // packages/propel/src/tooltip/root.tsx (mantener sincronizado si se
  // modifica uno).
  const safeChild = ((): React.ReactElement => {
    if (!React.isValidElement(children)) return children;
    const type = (children as React.ReactElement).type;
    if (typeof type === "string") return children;
    const FORWARD_REF = Symbol.for("react.forward_ref");
    const MEMO = Symbol.for("react.memo");
    const $$typeof = (type as unknown as { $$typeof?: symbol })?.$$typeof;
    if ($$typeof === FORWARD_REF) return children;
    if ($$typeof === MEMO) {
      const inner = (type as unknown as { type?: { $$typeof?: symbol } }).type;
      if (inner?.$$typeof === FORWARD_REF) return children;
    }
    if (typeof type === "function") {
      const proto = (type as unknown as { prototype?: { isReactComponent?: unknown } }).prototype;
      if (proto?.isReactComponent) return children;
      return <span className="contents">{children}</span>;
    }
    return children;
  })();

  if (!shouldRender) {
    return (
      <div ref={toolTipRef} className="flex h-full items-center">
        {safeChild}
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
        React.cloneElement(safeChild, {
          ref: eleReference,
          ...tooltipProps,
          ...safeChild.props,
        })
      }
    />
  );
}
