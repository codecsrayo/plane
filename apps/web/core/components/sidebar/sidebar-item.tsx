/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import Link from "next/link";
import { cn } from "@plane/utils";

// ============================================================================
// TYPES
// ============================================================================

interface AppSidebarItemData {
  href?: string;
  label?: string;
  icon?: React.ReactNode;
  isActive?: boolean;
  onClick?: () => void;
  disabled?: boolean;
  showLabel?: boolean;
}

interface AppSidebarItemProps {
  variant?: "link" | "button";
  item?: AppSidebarItemData;
  /**
   * Cuando `variant="button"` y el consumer ya envuelve este item en un
   * elemento interactivo (ej. `<Menu.Button>` de Headless UI), pasar
   * `as="div"` para evitar el warning `<button>` dentro de `<button>`.
   */
  as?: "button" | "div";
}

interface AppSidebarItemLabelProps {
  highlight?: boolean;
  label?: string;
}

interface AppSidebarItemIconProps {
  icon?: React.ReactNode;
  highlight?: boolean;
}

interface AppSidebarLinkItemProps {
  href?: string;
  children: React.ReactNode;
  className?: string;
}

interface AppSidebarButtonItemProps {
  children: React.ReactNode;
  onClick?: () => void;
  disabled?: boolean;
  className?: string;
  /**
   * Elemento HTML a renderizar. Default: `"button"`.
   *
   * Usar `"div"` cuando este componente se usa como `customButton` /
   * `Menu.Button` de Headless UI o similar — esos wrappers YA renderizan
   * un `<button>` y anidar otro dispara:
   *   "Warning: validateDOMNesting(…): <button> cannot appear as a
   *    descendant of <button>."
   */
  as?: "button" | "div";
}

// ============================================================================
// STYLES
// ============================================================================

const styles = {
  base: "group flex flex-col gap-0.5 items-center justify-center text-tertiary",
  icon: "flex items-center justify-center gap-2 size-8 rounded-md text-tertiary",
  iconActive: "bg-layer-transparent-selected text-secondary !text-icon-primary",
  iconInactive: "group-hover:text-icon-secondary group-hover:bg-layer-transparent-hover !text-icon-tertiary",
  label: "text-11 font-medium",
  labelActive: "text-secondary",
  labelInactive: "group-hover:text-secondary text-tertiary",
} as const;

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

function AppSidebarItemLabel({ highlight = false, label }: AppSidebarItemLabelProps) {
  if (!label) return null;

  return (
    <span
      className={cn(styles.label, {
        [styles.labelActive]: highlight,
        [styles.labelInactive]: !highlight,
      })}
    >
      {label}
    </span>
  );
}

function AppSidebarItemIcon({ icon, highlight }: AppSidebarItemIconProps) {
  if (!icon) return null;

  return (
    <div
      className={cn(styles.icon, {
        [styles.iconActive]: highlight,
        [styles.iconInactive]: !highlight,
      })}
    >
      {icon}
    </div>
  );
}

function AppSidebarLinkItem({ href, children, className }: AppSidebarLinkItemProps) {
  if (!href) return null;

  return (
    <Link href={href} className={cn(styles.base, className)}>
      {children}
    </Link>
  );
}

function AppSidebarButtonItem({
  children,
  onClick,
  disabled = false,
  className,
  as = "button",
}: AppSidebarButtonItemProps) {
  // Cuando `as="div"` el caller es responsable del comportamiento de boton
  // (lo tipico: un <Menu.Button> de Headless UI que envuelve a este
  // componente y ya aporta onClick/keyboard/aria). Aqui solo emitimos un
  // contenedor para los hijos — evita el warning validateDOMNesting de
  // <button> dentro de <button>.
  if (as === "div") {
    return (
      <div className={cn(styles.base, className)} aria-disabled={disabled || undefined}>
        {children}
      </div>
    );
  }
  return (
    <button className={cn(styles.base, className)} onClick={onClick} disabled={disabled} type="button">
      {children}
    </button>
  );
}

// ============================================================================
// MAIN COMPONENT
// ============================================================================

export type AppSidebarItemComponent = React.FC<AppSidebarItemProps> & {
  Label: React.FC<AppSidebarItemLabelProps>;
  Icon: React.FC<AppSidebarItemIconProps>;
  Link: React.FC<AppSidebarLinkItemProps>;
  Button: React.FC<AppSidebarButtonItemProps>;
};

// Nota: el `= {}` final es defensivo. Un `function Foo({ a = 1 })` crashea si
// se invoca con `undefined` como primer argumento — los defaults por propiedad
// no cubren el caso del objeto entero. El codemod WEB-5459 convirtió esto de
// arrow-function a function-declaration; mantener el fallback explícito evita
// el "Cannot read properties of undefined (reading 'variant')" si algún consumer
// pasa `undefined` (p.ej. via HMR, HOC con props mal tipados, o render-as-value).
function AppSidebarItem({ variant = "link", item, as }: AppSidebarItemProps = {}) {
  if (!item) return null;

  const { icon, isActive, label, href, onClick, disabled, showLabel = true } = item;

  const commonItems = (
    <>
      <AppSidebarItemIcon icon={icon} highlight={isActive} />
      {showLabel && <AppSidebarItemLabel highlight={isActive} label={label} />}
    </>
  );

  if (variant === "link") {
    return <AppSidebarLinkItem href={href}>{commonItems}</AppSidebarLinkItem>;
  }

  return (
    <AppSidebarButtonItem onClick={onClick} disabled={disabled} as={as}>
      {commonItems}
    </AppSidebarButtonItem>
  );
}

// ============================================================================
// COMPOUND COMPONENT ASSIGNMENT
// ============================================================================

AppSidebarItem.Label = AppSidebarItemLabel;
AppSidebarItem.Icon = AppSidebarItemIcon;
AppSidebarItem.Link = AppSidebarLinkItem;
AppSidebarItem.Button = AppSidebarButtonItem;

export { AppSidebarItem };
export type { AppSidebarItemData, AppSidebarItemProps };
