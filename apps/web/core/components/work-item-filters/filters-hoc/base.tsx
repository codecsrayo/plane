/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useEffect, useLayoutEffect, useMemo } from "react";
import { observer } from "mobx-react";
import { v4 as uuidv4 } from "uuid";
// plane imports
import type { TSaveViewOptions, TUpdateViewOptions } from "@plane/constants";
import type { IWorkItemFilterInstance } from "@plane/shared-state";
import type { IIssueFilters, TWorkItemFilterExpression } from "@plane/types";
// store hooks
import { useWorkItemFilters } from "@/hooks/store/work-item-filters/use-work-item-filters";
// plane web imports
import type { TWorkItemFiltersEntityProps } from "@/plane-web/hooks/work-item-filters/use-work-item-filters-config";
import { useWorkItemFiltersConfig } from "@/plane-web/hooks/work-item-filters/use-work-item-filters-config";
// local imports
import type { TSharedWorkItemFiltersHOCProps, TSharedWorkItemFiltersProps } from "./shared";

type TAdditionalWorkItemFiltersProps = {
  saveViewOptions?: TSaveViewOptions<TWorkItemFilterExpression>;
  updateViewOptions?: TUpdateViewOptions<TWorkItemFilterExpression>;
} & TWorkItemFiltersEntityProps;

type TWorkItemFiltersHOCProps = TSharedWorkItemFiltersHOCProps & TAdditionalWorkItemFiltersProps;

export const WorkItemFiltersHOC = observer(function WorkItemFiltersHOC(props: TWorkItemFiltersHOCProps) {
  const { children, initialWorkItemFilters } = props;

  // Only initialize filter instance when initial work item filters are defined
  if (!initialWorkItemFilters)
    return <>{typeof children === "function" ? children({ filter: undefined }) : children}</>;

  return (
    <WorkItemFilterRoot {...props} initialWorkItemFilters={initialWorkItemFilters}>
      {children}
    </WorkItemFilterRoot>
  );
});

type TWorkItemFilterProps = TSharedWorkItemFiltersProps &
  TAdditionalWorkItemFiltersProps & {
    initialWorkItemFilters: IIssueFilters;
    // `filter` puede ser undefined en el render inicial: la creación del
    // FilterInstance ocurre en useLayoutEffect (post-commit) para no escribir
    // en el observable store durante el render y evitar el warning de React
    // "Cannot update a component while rendering a different component" —
    // típico cuando otro observer (p.ej. WorkItemFiltersToggle en el header)
    // está suscrito a `getFilter` sobre el mismo entity. Todos los call sites
    // (project-layout-root, cycle-layout-root, etc.) ya aplican `filter && …`
    // defensivamente, así que widen-ear el tipo es no-breaking.
    children: React.ReactNode | ((props: { filter: IWorkItemFilterInstance | undefined }) => React.ReactNode);
  };

const WorkItemFilterRoot = observer(function WorkItemFilterRoot(props: TWorkItemFilterProps) {
  const {
    children,
    entityType,
    entityId,
    filtersToShowByLayout,
    initialWorkItemFilters,
    isTemporary,
    saveViewOptions,
    updateFilters,
    updateViewOptions,
    showOnMount,
    ...entityConfigProps
  } = props;
  // store hooks
  const { getFilter, getOrCreateFilter, deleteFilter } = useWorkItemFilters();
  // derived values
  const workItemEntityID = useMemo(
    () => (isTemporary ? `TEMP-${entityId ?? uuidv4()}` : entityId),
    [isTemporary, entityId]
  );
  // memoize initial values to prevent re-computations when reference changes
  const initialUserFilters = useMemo(() => initialWorkItemFilters.richFilters, [initialWorkItemFilters]);
  const workItemFiltersConfig = useWorkItemFiltersConfig({
    allowedFilters: filtersToShowByLayout ? filtersToShowByLayout : [],
    ...entityConfigProps,
  });

  // Lectura reactiva del filter instance. `getFilter` es un computedFn que
  // lee `filters.get(...)` del observable Map; suscribe a este observer a
  // cambios en ese slot sin mutarlo, por lo que es seguro en render.
  const workItemLayoutFilter = getFilter(entityType, workItemEntityID);

  // Sincroniza el FilterInstance dentro del store DESPUÉS del commit.
  // Antipatrón anterior: se llamaba `getOrCreateFilter` dentro de `useMemo`
  // (durante render) — esa action muta `this.filters.set(...)` y además,
  // en la rama "existing filter", asigna `onExpressionChange`,
  // `updateExpressionOptions` y `toggleVisibility` sobre campos observables
  // del FilterInstance. Cualquier otro observer suscrito via `getFilter`
  // (p.ej. WorkItemFiltersToggle en el header) es re-programado por MobX a
  // mitad del render de WorkItemFilterRoot → warning de React:
  // "Cannot update a component (`WorkItemFiltersToggle`) while rendering a
  // different component (`WorkItemFilterRoot`)".
  //
  // `useLayoutEffect` corre sincrónicamente post-commit y antes del paint,
  // así que el render inicial con `filter === undefined` no genera flash
  // visible: MobX notifica a los observers suscritos inmediatamente y
  // React re-renderiza con el filter disponible en el mismo frame.
  useLayoutEffect(
    () => {
      getOrCreateFilter({
        entityType,
        entityId: workItemEntityID,
        initialExpression: initialUserFilters,
        onExpressionChange: updateFilters,
        expressionOptions: {
          saveViewOptions,
          updateViewOptions,
        },
        showOnMount,
      });
    },
    // Mismas deps que el useMemo original — `initialUserFilters` y
    // `showOnMount` se mantienen fuera a propósito: sólo aplican a la
    // creación inicial y no deben recrear/ocultar un filter ya visible que
    // el usuario haya interactuado.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [entityType, workItemEntityID, saveViewOptions, updateViewOptions, updateFilters]
  );

  // delete filter instance when component unmounts
  useEffect(
    () => () => {
      deleteFilter(entityType, workItemEntityID);
    },
    [deleteFilter, entityType, workItemEntityID]
  );

  useEffect(() => {
    // En el primer render `workItemLayoutFilter` aún es undefined porque la
    // creación se difiere a useLayoutEffect. Este effect se vuelve a disparar
    // cuando la instancia aparece (cambio de identidad en deps).
    if (!workItemLayoutFilter) return;
    workItemLayoutFilter.configManager.setAreConfigsReady(workItemFiltersConfig.areAllConfigsInitialized);
    workItemLayoutFilter.configManager.registerAll(workItemFiltersConfig.configs);
  }, [workItemLayoutFilter, workItemFiltersConfig.areAllConfigsInitialized, workItemFiltersConfig.configs]);

  return <>{typeof children === "function" ? children({ filter: workItemLayoutFilter }) : children}</>;
});
