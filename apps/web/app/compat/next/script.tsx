/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useEffect, useMemo, type ScriptHTMLAttributes } from "react";

/**
 * Minimal shim for next/script.
 *
 * Only a known subset of <script> HTML attributes is accepted as extra props.
 * This is intentional: the shim does `script.setAttribute(key, value)` in a loop,
 * so an unbounded `[key: string]: any` pass-through would let callers inject
 * arbitrary DOM attributes (including event handlers like `onerror`). Keep the
 * whitelist tight; if a new attribute is needed, add it here explicitly.
 */
type AllowedExtraAttr =
  | "async"
  | "crossOrigin"
  | "defer"
  | "fetchPriority"
  | "integrity"
  | "nonce"
  | "noModule"
  | "referrerPolicy"
  | "type";

type ScriptProps = {
  src?: string;
  id?: string;
  strategy?: "beforeInteractive" | "afterInteractive" | "lazyOnload" | "worker";
  onLoad?: () => void;
  onError?: () => void;
  children?: string;
  defer?: boolean;
} & Pick<ScriptHTMLAttributes<HTMLScriptElement>, AllowedExtraAttr>;

// camelCase → lowercase DOM attribute (setAttribute expects lowercase HTML attrs).
const DOM_ATTR_NAME: Record<AllowedExtraAttr, string> = {
  async: "async",
  crossOrigin: "crossorigin",
  defer: "defer",
  fetchPriority: "fetchpriority",
  integrity: "integrity",
  nonce: "nonce",
  noModule: "nomodule",
  referrerPolicy: "referrerpolicy",
  type: "type",
};

function Script({ src, id, strategy: _strategy, onLoad, onError, children, ...rest }: ScriptProps) {
  // Stable key for the `rest` object so the effect doesn't re-run (and
  // re-insert the <script>) on every parent render just because `rest` is
  // a new object literal. JSON is safe here because values are primitives.
  const restKey = useMemo(() => JSON.stringify(rest ?? {}), [rest]);

  useEffect(() => {
    if (!src && !children) return;

    const script = document.createElement("script");
    if (id) script.id = id;
    if (src) script.src = src;
    if (children) script.textContent = children;
    if (onLoad) script.addEventListener("load", onLoad);
    if (onError) script.addEventListener("error", onError);

    // Only copy whitelisted attributes. Unknown keys are dropped — never
    // forwarded blindly via setAttribute.
    for (const key of Object.keys(rest) as AllowedExtraAttr[]) {
      const domName = DOM_ATTR_NAME[key];
      if (!domName) continue;
      const value = rest[key];
      if (value === undefined || value === null || value === false) continue;
      script.setAttribute(domName, value === true ? "" : String(value));
    }

    document.body.appendChild(script);

    return () => {
      if (onLoad) script.removeEventListener("load", onLoad);
      if (onError) script.removeEventListener("error", onError);
      script.remove();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- `restKey` covers `rest` stably
  }, [src, id, children, onLoad, onError, restKey]);

  return null;
}

export default Script;

