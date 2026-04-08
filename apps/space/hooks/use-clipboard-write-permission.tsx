/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useState, useEffect } from "react";

const useClipboardWritePermission = () => {
  const [isClipboardWriteAllowed, setClipboardWriteAllowed] = useState(false);

  useEffect(() => {
    const checkClipboardWriteAccess = async () => {
      try {
        const result = await navigator.permissions.query({ name: "clipboard-write" as PermissionName });
        setClipboardWriteAllowed(result.state === "granted");
      } catch {
        setClipboardWriteAllowed(false);
      }
    };

    void checkClipboardWriteAccess();
  }, []);

  return isClipboardWriteAllowed;
};

export default useClipboardWritePermission;
