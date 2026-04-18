/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useEffect, useMemo, useRef } from "react";
import { useLocation, useNavigate, useParams as useParamsRR, useSearchParams as useSearchParamsRR } from "react-router";
import { ensureTrailingSlash } from "./helper";

export function useRouter() {
  const navigate = useNavigate();
  // Track every deferred navigation timer so we can cancel pending ones on
  // unmount. Without this, `setTimeout(() => navigate(...), 0)` fires after
  // the calling component is gone and schedules a navigation anyway, which is
  // both a resource leak and a source of ghost navigations during fast unmount
  // (e.g. route transition after a click).
  const pendingTimers = useRef<Set<ReturnType<typeof setTimeout>>>(new Set());

  useEffect(
    () => () => {
      for (const id of pendingTimers.current) clearTimeout(id);
      pendingTimers.current.clear();
    },
    []
  );

  return useMemo(() => {
    const defer = (fn: () => void) => {
      const id = setTimeout(() => {
        pendingTimers.current.delete(id);
        fn();
      }, 0);
      pendingTimers.current.add(id);
    };

    return {
      // NOTE: the original bug this shim works around — calling `navigate()`
      // synchronously during render — should be fixed in the caller. Deferring
      // via setTimeout here papers over it. Remove this shim once callers are
      // migrated to react-router's `useNavigate` directly.
      push: (to: string) => defer(() => navigate(ensureTrailingSlash(to))),
      replace: (to: string) => defer(() => navigate(ensureTrailingSlash(to), { replace: true })),
      back: () => defer(() => navigate(-1)),
      forward: () => defer(() => navigate(1)),
      refresh: () => {
        // Guard for SSR / non-browser runtimes.
        if (typeof window !== "undefined") window.location.reload();
      },
      prefetch: async (_to: string) => {
        // no-op in this shim
      },
    };
  }, [navigate]);
}

export function usePathname(): string {
  const { pathname } = useLocation();
  return pathname;
}

export function useSearchParams(): URLSearchParams {
  const [searchParams] = useSearchParamsRR();
  return searchParams;
}

export function useParams() {
  return useParamsRR();
}

