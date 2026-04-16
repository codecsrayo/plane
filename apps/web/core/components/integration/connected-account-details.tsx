/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

type Props = {
  metadata: Record<string, unknown> | null;
};

export function ConnectedAccountDetails({ metadata }: Props) {
  if (!metadata || Object.keys(metadata).length === 0) {
    return <p className="text-sm text-custom-text-300">No account details available.</p>;
  }

  const rows: { label: string; value: string }[] = [];

  if (typeof metadata.installation_id !== "undefined") {
    rows.push({ label: "Installation ID", value: String(metadata.installation_id) });
  }
  if (typeof metadata.team_name !== "undefined") {
    rows.push({ label: "Team name", value: String(metadata.team_name) });
  }
  if (typeof metadata.team_id !== "undefined") {
    rows.push({ label: "Team ID", value: String(metadata.team_id) });
  }
  if (typeof metadata.account !== "undefined") {
    rows.push({ label: "Account", value: String(metadata.account) });
  }
  if (typeof metadata.login !== "undefined") {
    rows.push({ label: "Login", value: String(metadata.login) });
  }

  if (rows.length === 0) {
    for (const [key, value] of Object.entries(metadata)) {
      rows.push({ label: key, value: String(value) });
    }
  }

  return (
    <dl className="space-y-2">
      {rows.map(({ label, value }) => (
        <div key={label} className="flex items-center gap-3">
          <dt className="text-xs text-custom-text-200 w-36 shrink-0 font-medium">{label}</dt>
          <dd className="text-xs text-custom-text-100">{value}</dd>
        </div>
      ))}
    </dl>
  );
}
