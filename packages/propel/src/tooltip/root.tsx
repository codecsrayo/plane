/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";
import { Tooltip as BaseTooltip } from "@base-ui-components/react/tooltip";
import { cn } from "../utils";
import type { TPlacement, TSide, TAlign } from "../utils/placement";
import { convertPlacementToSideAndAlign } from "../utils/placement";

type ITooltipProps = {
  tooltipHeading?: string;
  tooltipContent?: string | React.ReactNode | null;
  position?: TPlacement;
  children: React.ReactElement;
  disabled?: boolean;
  className?: string;
  openDelay?: number;
  closeDelay?: number;
  isMobile?: boolean;
  renderByDefault?: boolean;
  side?: TSide;
  align?: TAlign;
  sideOffset?: number;
};

// Defensa en dos capas contra `props` inválido:
//
// 1. Default del parámetro (`= {} as ITooltipProps`): atrapa el caso en que el componente es
//    invocado con `undefined` (p.ej. `<Tooltip {...undefined} />`, render-as-value en libs
//    headless tipo base-ui donde un `render={Tooltip}` acaba llamando al componente sin
//    argumentos, o HMR/Fast Refresh swaps que filtran `undefined`). Los defaults por propiedad
//    NO cubren esto — si `props` entero es `undefined`, la destructuración crashea con
//    "Cannot destructure property 'tooltipHeading' of 'props' as it is undefined" antes de
//    que los defaults por-prop lleguen a aplicarse.
//
// 2. Nullish-coalesce dentro del cuerpo (`props ?? ({} as ITooltipProps)`): el default de
//    parámetro SOLO se dispara con `undefined`, no con `null`. Si un HOC o un render prop mal
//    tipado llega a pasar `null` explícito, la función recibiría `props === null` y la
//    destructuración volvería a crashear con "as it is null". El `??` absorbe ese caso.
//
// El cast a `ITooltipProps` es necesario porque `children` está declarado como requerido en
// el tipo; con un objeto realmente vacío el componente renderiza un provider sin contenido
// útil, pero no revienta la app — una mejora estricta sobre reventar el subtree de React.
// Mismo patrón aplicado en AppSidebarItem (commit 099091c) y ui Tooltip (commit 6aec132).
// Normaliza `children` para que BaseTooltip.Trigger pueda pasarle una ref:
//
// base-ui's <Tooltip.Trigger render={children}/> clona `children` y le asigna
// una ref. Si `children` es un function component sin React.forwardRef, React
// loguea:
//   "Warning: Function components cannot be given refs. Attempts to access
//    this ref will fail. Did you mean to use React.forwardRef()?
//    Check the render method of `TooltipTrigger`."
//
// El fix correcto seria que cada caller envolviera su function component en
// forwardRef, pero hay 184 usos de <Tooltip> en el repo — una migracion masiva
// es fragil. En su lugar, cuando detectamos que `children` es un function
// component (ni string DOM ni class ni forwardRef), lo envolvemos en un
// <span> inline que SI acepta ref. El <span> es transparente visualmente y
// mantiene las posiciones del tooltip.
function ensureRefCompatibleChild(children: React.ReactElement): React.ReactElement {
  if (!React.isValidElement(children)) return children;
  const type = (children as React.ReactElement).type;
  // Strings ("div", "button", etc.) y tags DOM siempre aceptan ref.
  if (typeof type === "string") return children;
  // forwardRef/memoForwardRef exponen $$typeof === Symbol(react.forward_ref).
  // Accedemos via any porque el tipo publico de React no expone $$typeof.
  const $$typeof = (type as unknown as { $$typeof?: symbol })?.$$typeof;
  const FORWARD_REF = Symbol.for("react.forward_ref");
  const MEMO = Symbol.for("react.memo");
  if ($$typeof === FORWARD_REF) return children;
  // memo puede envolver un forwardRef — profundizar una capa.
  if ($$typeof === MEMO) {
    const inner = (type as unknown as { type?: unknown }).type as
      | { $$typeof?: symbol }
      | undefined;
    if (inner?.$$typeof === FORWARD_REF) return children;
  }
  // Class components: function con prototype.isReactComponent.
  if (typeof type === "function") {
    const proto = (type as unknown as { prototype?: { isReactComponent?: unknown } }).prototype;
    if (proto?.isReactComponent) return children;
    // Function component sin forwardRef — envolvemos en <span>.
    return <span className="contents">{children}</span>;
  }
  return children;
}

export function Tooltip(props: ITooltipProps = {} as ITooltipProps) {
  const {
    tooltipHeading,
    tooltipContent,
    position = "top",
    children,
    disabled = false,
    className = "",
    openDelay = 200,
    side = "bottom",
    align = "center",
    sideOffset = 10,
    closeDelay,
    isMobile = false,
  } = props ?? ({} as ITooltipProps);
  const { finalSide, finalAlign } = React.useMemo(() => {
    if (position) {
      const converted = convertPlacementToSideAndAlign(position);
      return { finalSide: converted.side, finalAlign: converted.align };
    }
    return { finalSide: side, finalAlign: align };
  }, [position, side, align]);

  // Guard: si children no es un elemento valido (null/undefined/string/number),
  // no hay anchor para el tooltip. Renderizamos children tal cual sin el
  // provider — evita reventar base-ui con 'render' no-element.
  if (!React.isValidElement(children)) return <>{children}</>;

  const safeChild = ensureRefCompatibleChild(children);

  return (
    <BaseTooltip.Provider>
      <BaseTooltip.Root delay={openDelay} closeDelay={closeDelay} disabled={disabled}>
        <BaseTooltip.Trigger render={safeChild} />
        <BaseTooltip.Portal>
          <BaseTooltip.Positioner
            className={cn(
              "z-50 max-w-xs gap-1 overflow-hidden rounded-lg border border-subtle-1 bg-layer-2 px-2 py-1.5 break-words shadow-overlay-200",
              {
                hidden: isMobile,
              },
              className
            )}
            side={finalSide}
            sideOffset={sideOffset}
            align={finalAlign}
            render={
              <BaseTooltip.Popup>
                {tooltipHeading && <p className="text-caption-md-medium text-primary">{tooltipHeading}</p>}
                {tooltipContent && (
                  <p
                    className={cn("text-caption-sm-regular text-secondary", {
                      "mt-1": tooltipHeading && tooltipHeading !== "",
                    })}
                  >
                    {tooltipContent}
                  </p>
                )}
              </BaseTooltip.Popup>
            }
          />
        </BaseTooltip.Portal>
      </BaseTooltip.Root>
    </BaseTooltip.Provider>
  );
}
