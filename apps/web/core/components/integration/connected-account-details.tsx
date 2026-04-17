/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

type Props = {
  metadata: Record<string, unknown> | null;
};

/**
 * Known account metadata keys with human-friendly labels, in display order.
 *
 * Declared as a `readonly` tuple so the iteration order is stable and new
 * providers (Slack, Jira, …) only need a single-line addition here rather
 * than another `if (typeof metadata.X !== "undefined")` branch.
 */
const KNOWN_METADATA_FIELDS = [
  { key: "installation_id", label: "Installation ID" },
  { key: "team_name", label: "Team name" },
  { key: "team_id", label: "Team ID" },
  { key: "account", label: "Account" },
  { key: "login", label: "Login" },
] as const;

export function ConnectedAccountDetails({ metadata }: Props) {
  if (!metadata || Object.keys(metadata).length === 0) {
    return <p className="text-sm text-custom-text-300">No account details available.</p>;
  }

  // `!= null` (loose) also rejects `undefined`; the previous
  // `typeof x !== "undefined"` check let `null` through because
  // `typeof null === "object"`, which rendered the literal string "null".
  const rows: { label: string; value: string }[] = KNOWN_METADATA_FIELDS.flatMap(({ key, label }) => {
    const value = metadata[key];
    return value != null ? [{ label, value: String(value) }] : [];
  });

  // Fallback: if none of the well-known keys matched, show whatever
  // the backend returned — still filtering nullish values.
  if (rows.length === 0) {
    for (const [key, value] of Object.entries(metadata)) {
      if (value != null) rows.push({ label: key, value: String(value) });
    }
  }

  if (rows.length === 0) {
    return <p className="text-sm text-custom-text-300">No account details available.</p>;
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
