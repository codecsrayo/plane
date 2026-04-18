/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import type { Control, FieldValues } from "react-hook-form";

/**
 * CE stub for the AccessController. In CE builds this renders nothing —
 * access-level control is an EE feature. The real implementation lives in
 * `plane-web/components/views/access-controller`.
 *
 * The generic `control` prop is typed (not `any`) so that callers keep
 * their form-type inference; the CE override silently drops the value.
 */
type AccessControllerProps<T extends FieldValues> = {
  control: Control<T>;
};

export function AccessController<T extends FieldValues>(_props: AccessControllerProps<T>) {
  return <></>;
}
