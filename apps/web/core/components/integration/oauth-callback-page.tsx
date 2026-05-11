/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 *
 * Shared OAuth callback UI shell used by GitHub, GitLab and Slack callback pages.
 *
 * All three providers share identical UI structure — a centred card with three
 * mutually exclusive states (processing / success / error). This component
 * owns that UI so the per-provider pages only need to supply the status,
 * error message and provider-specific copy.
 */

import { type FC } from "react";

export type TOAuthCallbackStatus = "processing" | "success" | "error";

interface OAuthCallbackPageProps {
  /** Current state of the OAuth exchange. */
  status: TOAuthCallbackStatus;
  /** Error message shown when status === "error". */
  errorMessage: string;
  /** Provider-specific copy shown during the loading state. */
  processingText: string;
  /** Provider-specific copy shown on success. */
  successText: string;
}

/**
 * Stateless display-only component — all logic lives in the parent page.
 */
export const OAuthCallbackPage: FC<OAuthCallbackPageProps> = ({
  status,
  errorMessage,
  processingText,
  successText,
}) => (
  <div className="flex h-screen w-full items-center justify-center bg-surface-1">
    <div className="shadow-sm flex flex-col items-center gap-4 rounded-lg border border-subtle bg-surface-2 p-10">
      {status === "processing" && (
        <>
          <div className="border-primary-400 size-8 animate-spin rounded-full border-4 border-t-transparent" />
          <p className="text-body-sm-medium text-secondary">{processingText}</p>
        </>
      )}

      {status === "success" && (
        <>
          <svg className="size-12 text-success-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
          </svg>
          <p className="text-body-sm-medium">{successText}</p>
        </>
      )}

      {status === "error" && (
        <>
          <svg className="text-red-500 size-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
          <p className="text-red-500 text-body-sm-medium">{errorMessage}</p>
          <button className="text-sm mt-2 text-secondary underline" onClick={() => window.close()}>
            Close this window
          </button>
        </>
      )}
    </div>
  </div>
);
