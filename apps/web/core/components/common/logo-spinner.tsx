/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useEffect, useState } from "react";
import { useTheme } from "next-themes";
// assets
import LogoSpinnerDark from "@/app/assets/images/logo-spinner-dark.gif?url";
import LogoSpinnerLight from "@/app/assets/images/logo-spinner-light.gif?url";

// Paridad entre server render y primer client render:
//
// `useTheme()` de next-themes retorna `resolvedTheme === undefined` en SSR y en el
// primer render del cliente (lee del DOM/localStorage sólo dentro de un useEffect).
// Si renderizamos el <img> inmediatamente, el primer paint del client puede diferir
// del HTML del server porque el `<img src>` depende del tema — provoca:
//
//   "Warning: Expected server HTML to contain a matching <div> in <div>"
//   "Error: Hydration failed because the initial UI does not match…"
//
// Solución estándar para componentes theme-aware en SSR: diferir el render del
// contenido theme-aware hasta después del primer commit del cliente (mounted=true).
// Antes de eso, emitimos exactamente el mismo shell que el server para que la
// hidratación pase en silencio.
export function LogoSpinner() {
  const { resolvedTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  // Shell estable: misma estructura en server y en el primer paint del cliente.
  // El src se fija a la variante light como placeholder determinístico; en la
  // práctica no se ve porque el div se remonta en el siguiente tick con el
  // tema real. Nunca dejamos el <img> sin src para no disparar una request 404.
  const logoSrc = mounted && resolvedTheme === "dark" ? LogoSpinnerDark : LogoSpinnerLight;

  return (
    <div className="flex items-center justify-center">
      <img src={logoSrc} alt="logo" className="h-6 w-auto object-contain sm:h-11" />
    </div>
  );
}
