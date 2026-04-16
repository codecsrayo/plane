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

  return (
    <BaseTooltip.Provider>
      <BaseTooltip.Root delay={openDelay} closeDelay={closeDelay} disabled={disabled}>
        <BaseTooltip.Trigger render={children} />
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
