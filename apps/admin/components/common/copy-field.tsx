/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import React from "react";
// ui
import { Button } from "@plane/propel/button";
import { CheckIcon, CopyIcon } from "@plane/propel/icons";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";

type Props = {
  label: string;
  url: string;
  description: string | React.ReactNode;
};

export type TCopyField = {
  key: string;
  label: string;
  url: string;
  description: string | React.ReactNode;
};

export function CopyField(props: Props) {
  const { label, url, description } = props;
  const [isCopied, setIsCopied] = React.useState(false);
  const timeoutRef = React.useRef<ReturnType<typeof setTimeout> | null>(null);

  React.useEffect(
    () => () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    },
    []
  );

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(url);
      setIsCopied(true);
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
      timeoutRef.current = setTimeout(() => {
        setIsCopied(false);
        timeoutRef.current = null;
      }, 2000);
    } catch {
      setIsCopied(false);
      setToast({
        type: TOAST_TYPE.ERROR,
        title: "Copy failed",
        message: "Could not copy to clipboard. Please copy the value manually.",
      });
    }
  };

  return (
    <div className="flex flex-col gap-1">
      <h4 className="text-13 text-secondary">{label}</h4>
      <Button variant="secondary" size="lg" className="flex items-center justify-between py-2" onClick={handleCopy}>
        <p className="text-13 font-medium">{url}</p>
        {isCopied ? (
          // text-success-primary resolves via currentColor — respects dark mode and theming
          <CheckIcon width={18} height={18} className="text-success-primary" />
        ) : (
          // text-tertiary matches the muted icon token used across the admin UI
          <CopyIcon width={18} height={18} className="text-tertiary" />
        )}
      </Button>
      <div className="text-11 text-tertiary">{description}</div>
    </div>
  );
}
