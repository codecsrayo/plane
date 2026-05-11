/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 *
 * GitHub App post-installation setup redirect.
 *
 * GitHub uses this as the "Setup URL" after an app is installed or re-configured:
 *   /auth/github/setup?installation_id=XXXXX&setup_action=install&state={workspaceSlug}
 *
 * This page simply normalizes the params and redirects to the real callback page
 * so that both install and update flows share the same processing logic.
 */

import { useEffect } from "react";
import { useNavigate, useSearchParams } from "react-router";

export default function GithubSetupRedirectPage() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();

  useEffect(() => {
    const installation_id = searchParams.get("installation_id");
    const setup_action = searchParams.get("setup_action") ?? "update";
    const state = searchParams.get("state");

    const params = new URLSearchParams();
    if (installation_id) params.set("installation_id", installation_id);
    if (setup_action) params.set("setup_action", setup_action);
    if (state) params.set("state", state);

    navigate(`/auth/github/callback?${params.toString()}`, { replace: true });
  }, [navigate, searchParams]);

  return (
    <div className="flex h-screen w-full items-center justify-center bg-surface-1">
      <div className="flex flex-col items-center gap-4">
        <div className="border-primary-400 size-8 animate-spin rounded-full border-4 border-t-transparent" />
        <p className="text-body-sm-medium text-secondary">Redirecting…</p>
      </div>
    </div>
  );
}
