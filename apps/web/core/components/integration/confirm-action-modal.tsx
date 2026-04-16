/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import type { ReactNode } from "react";
import { AlertModalCore } from "@plane/ui";

type Props = {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => Promise<void>;
  isSubmitting: boolean;
  title: string;
  content: ReactNode;
};

export function IntegrationConfirmActionModal({ isOpen, onClose, onConfirm, isSubmitting, title, content }: Props) {
  return (
    <AlertModalCore
      handleClose={onClose}
      handleSubmit={onConfirm}
      isSubmitting={isSubmitting}
      isOpen={isOpen}
      title={title}
      content={content}
    />
  );
}
