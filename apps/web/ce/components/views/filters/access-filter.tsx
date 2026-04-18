/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

type FilterByAccessProps = {
  appliedFilters: string[] | null | undefined;
  handleUpdate: (val: string | string[]) => void;
  searchQuery: string;
  accessFilters: { key: string; value: string }[];
};

// CE stub: access filter is a commercial-edition feature. Accept the same prop
// shape the core filter panel passes, but render nothing.
// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function FilterByAccess(_props: FilterByAccessProps) {
  return <></>;
}
