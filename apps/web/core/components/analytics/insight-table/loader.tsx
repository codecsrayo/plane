/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import * as React from "react";
import type { ColumnDef } from "@tanstack/react-table";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@plane/propel/table";
import { Loader } from "@plane/ui";

interface TableSkeletonProps {
  columns: ColumnDef<any>[];
  rows: number;
}

const getColumnKey = (column: ColumnDef<any>) => String(column.id ?? column.accessorKey ?? column.header ?? "column");

export function TableLoader({ columns, rows }: TableSkeletonProps) {
  const rowKeys = Array.from({ length: rows }, (_, rowIndex) => `row-${rowIndex}`);

  return (
    <Table>
      <TableHeader>
        <TableRow>
          {columns.map((column) => (
            <TableHead key={getColumnKey(column)}>{typeof column.header === "string" ? column.header : ""}</TableHead>
          ))}
        </TableRow>
      </TableHeader>
      <TableBody>
        {rowKeys.map((rowKey) => (
          <TableRow key={rowKey}>
            {columns.map((column) => (
              <TableCell key={`${rowKey}-${getColumnKey(column)}`}>
                <Loader.Item height="20px" width="100%" />
              </TableCell>
            ))}
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
