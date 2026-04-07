/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useState } from "react";
import Link from "next/link";
import { observer } from "mobx-react";
import { Copy } from "lucide-react";
import { useForm } from "react-hook-form";
// plane internal packages
import { Button, getButtonStyling } from "@plane/propel/button";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
import type {
  IFormattedInstanceConfiguration,
  TInstanceAuthenticationMethodKeys,
  TInstanceIntegrationConfigurationKeys,
} from "@plane/types";
import { ToggleSwitch } from "@plane/ui";
// components
import { CodeBlock } from "@/components/common/code-block";
import { ConfirmDiscardModal } from "@/components/common/confirm-discard-modal";
import type { TControllerInputFormField } from "@/components/common/controller-input";
import { ControllerInput } from "@/components/common/controller-input";
// hooks
import { useInstance } from "@/hooks/store";

type Props = {
  config: IFormattedInstanceConfiguration;
};

type IntegrationConfigFormValues = Record<TInstanceIntegrationConfigurationKeys, string>;

// ─── Section component ────────────────────────────────────────────────────────
function Section({
  title,
  description,
  fields,
  control,
  errors,
  toggleKey,
  enabled,
  onToggle,
}: {
  title: string;
  description: React.ReactNode;
  fields: TControllerInputFormField[];
  control: any;
  errors: any;
  toggleKey?: TInstanceAuthenticationMethodKeys;
  enabled?: boolean;
  onToggle?: () => void;
}) {
  return (
    <div className="flex flex-col gap-4 border-b border-subtle pb-8 last:border-none">
      <div className="flex items-start justify-between gap-4">
        <div className="flex flex-col gap-1">
          <div className="text-lg font-medium">{title}</div>
          <p className="text-sm text-secondary">{description}</p>
        </div>
        {toggleKey !== undefined && onToggle !== undefined && (
          <div className="flex shrink-0 items-center gap-2 pt-1">
            <span className="text-sm text-secondary">{enabled ? "Enabled" : "Disabled"}</span>
            <ToggleSwitch value={Boolean(enabled)} onChange={onToggle} size="sm" />
          </div>
        )}
      </div>
      <div className={enabled === false ? "opacity-50" : undefined}>
        {fields.map((field) => (
          <div key={field.key} className="mb-4 last:mb-0">
            <ControllerInput
              control={control}
              type={field.type}
              name={field.key}
              label={field.label}
              description={field.description}
              placeholder={field.placeholder}
              error={Boolean(errors[field.key])}
              required={enabled === false ? false : field.required}
              disabled={enabled === false}
            />
          </div>
        ))}
      </div>
    </div>
  );
}

// ─── Main form ────────────────────────────────────────────────────────────────
export const InstanceIntegrationsConfigForm = observer(function InstanceIntegrationsConfigForm({ config }: Props) {
  const [isDiscardChangesModalOpen, setIsDiscardChangesModalOpen] = useState(false);
  const { formattedConfig, updateInstanceConfigurations } = useInstance();

  // ── Toggle helpers ─────────────────────────────────────────────────────────
  const isGithubEnabled = Boolean(parseInt(formattedConfig?.IS_GITHUB_ENABLED ?? "0"));
  const isGitlabEnabled = Boolean(parseInt(formattedConfig?.IS_GITLAB_ENABLED ?? "0"));
  const isSlackEnabled = Boolean(parseInt(formattedConfig?.IS_SLACK_ENABLED ?? "0"));

  const handleToggle = (key: TInstanceAuthenticationMethodKeys, current: boolean) => {
    updateInstanceConfigurations({ [key]: current ? "0" : "1" } as Record<string, string>).catch(console.error);
  };

  const origin = typeof window !== "undefined" ? window.location.origin : "";

  const {
    handleSubmit,
    control,
    reset,
    setValue,
    formState: { errors, isDirty, isSubmitting },
  } = useForm<IntegrationConfigFormValues>({
    defaultValues: {
      GITHUB_APP_NAME: config["GITHUB_APP_NAME"] ?? "",
      GITHUB_APP_ID: config["GITHUB_APP_ID"] ?? "",
      GITHUB_APP_PRIVATE_KEY: config["GITHUB_APP_PRIVATE_KEY"] ?? "",
      GITHUB_WEBHOOK_SECRET: config["GITHUB_WEBHOOK_SECRET"] ?? "",
      SLACK_CLIENT_ID: config["SLACK_CLIENT_ID"] ?? "",
      SLACK_CLIENT_SECRET: config["SLACK_CLIENT_SECRET"] ?? "",
      GITLAB_HOST: config["GITLAB_HOST"] ?? "https://gitlab.com",
      GITLAB_CLIENT_ID: config["GITLAB_CLIENT_ID"] ?? "",
      GITLAB_CLIENT_SECRET: config["GITLAB_CLIENT_SECRET"] ?? "",
    },
  });

  // ── GitHub helpers ─────────────────────────────────────────────────────────
  const generateWebhookSecret = () => {
    const secret = Array.from(crypto.getRandomValues(new Uint8Array(32)))
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
    setValue("GITHUB_WEBHOOK_SECRET", secret, { shouldDirty: true });
  };

  // ── GitHub fields ──────────────────────────────────────────────────────────
  const githubFields: TControllerInputFormField[] = [
    {
      key: "GITHUB_APP_NAME",
      type: "text",
      label: "GitHub App name",
      description: (
        <>
          Slug of your{" "}
          <a
            href="https://github.com/settings/apps"
            target="_blank"
            rel="noreferrer"
            className="text-accent-primary hover:underline"
          >
            GitHub App
          </a>
          . Appears in the install URL:{" "}
          <CodeBlock darkerShade>github.com/apps/&lt;THIS-NAME&gt;/installations/new</CodeBlock>
        </>
      ),
      placeholder: "your-github-app-name",
      error: Boolean(errors.GITHUB_APP_NAME),
      required: true,
    },
    {
      key: "GITHUB_APP_ID",
      type: "text",
      label: "App ID",
      description: (
        <>
          Numeric ID of your GitHub App. Found in{" "}
          <a
            href="https://github.com/settings/apps"
            target="_blank"
            rel="noreferrer"
            className="text-accent-primary hover:underline"
          >
            GitHub App settings
          </a>{" "}
          under <CodeBlock darkerShade>General → About → App ID</CodeBlock>.
        </>
      ),
      placeholder: "123456",
      error: Boolean(errors.GITHUB_APP_ID),
      required: true,
    },
    {
      key: "GITHUB_APP_PRIVATE_KEY",
      type: "password",
      label: "Private key (base64)",
      description: (
        <>
          Generate a private key in your GitHub App settings and encode it:{" "}
          <CodeBlock darkerShade>base64 -w0 private-key.pem</CodeBlock>. Required for installation access tokens.
        </>
      ),
      placeholder: "LS0tLS1CRUdJTi...",
      error: Boolean(errors.GITHUB_APP_PRIVATE_KEY),
      required: true,
    },
    {
      key: "GITHUB_WEBHOOK_SECRET",
      type: "password",
      label: "Webhook secret",
      description: "Used to verify webhook payloads from GitHub. Generate one or set your own.",
      placeholder: "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
      error: Boolean(errors.GITHUB_WEBHOOK_SECRET),
      required: false,
    },
  ];

  // ── GitLab fields ──────────────────────────────────────────────────────────
  const gitlabFields: TControllerInputFormField[] = [
    {
      key: "GITLAB_HOST",
      type: "text",
      label: "GitLab host",
      description: (
        <>
          Use <CodeBlock darkerShade>https://gitlab.com</CodeBlock> for GitLab.com or your self-hosted URL.
        </>
      ),
      placeholder: "https://gitlab.com",
      error: Boolean(errors.GITLAB_HOST),
      required: true,
    },
    {
      key: "GITLAB_CLIENT_ID",
      type: "text",
      label: "Application ID",
      description: (
        <>
          Found in your{" "}
          <a
            href="https://gitlab.com/-/profile/applications"
            target="_blank"
            rel="noreferrer"
            className="text-accent-primary hover:underline"
          >
            GitLab OAuth Application
          </a>{" "}
          settings.
        </>
      ),
      placeholder: "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
      error: Boolean(errors.GITLAB_CLIENT_ID),
      required: true,
    },
    {
      key: "GITLAB_CLIENT_SECRET",
      type: "password",
      label: "Secret",
      description: <>Secret from the same GitLab OAuth Application settings page.</>,
      placeholder: "gloas-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
      error: Boolean(errors.GITLAB_CLIENT_SECRET),
      required: true,
    },
  ];

  // ── Slack fields ───────────────────────────────────────────────────────────
  const slackFields: TControllerInputFormField[] = [
    {
      key: "SLACK_CLIENT_ID",
      type: "text",
      label: "Client ID",
      description: (
        <>
          Found in your{" "}
          <a
            href="https://api.slack.com/apps"
            target="_blank"
            rel="noreferrer"
            className="text-accent-primary hover:underline"
          >
            Slack App
          </a>{" "}
          under Basic Information → App Credentials.
        </>
      ),
      placeholder: "1234567890.1234567890123",
      error: Boolean(errors.SLACK_CLIENT_ID),
      required: false,
    },
    {
      key: "SLACK_CLIENT_SECRET",
      type: "password",
      label: "Client secret",
      description: <>Found in the same Slack App credentials page.</>,
      placeholder: "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
      error: Boolean(errors.SLACK_CLIENT_SECRET),
      required: false,
    },
  ];

  // ── Submit ─────────────────────────────────────────────────────────────────
  const onSubmit = async (formData: IntegrationConfigFormValues) => {
    try {
      const response = await updateInstanceConfigurations({ ...formData });
      setToast({
        type: TOAST_TYPE.SUCCESS,
        title: "Saved!",
        message: "Integration settings updated successfully.",
      });
      const get = (key: TInstanceIntegrationConfigurationKeys) => response.find((i) => i.key === key)?.value ?? "";
      reset({
        GITHUB_APP_NAME: get("GITHUB_APP_NAME"),
        GITHUB_APP_ID: get("GITHUB_APP_ID"),
        GITHUB_APP_PRIVATE_KEY: get("GITHUB_APP_PRIVATE_KEY"),
        GITHUB_WEBHOOK_SECRET: get("GITHUB_WEBHOOK_SECRET"),
        SLACK_CLIENT_ID: get("SLACK_CLIENT_ID"),
        SLACK_CLIENT_SECRET: get("SLACK_CLIENT_SECRET"),
        GITLAB_HOST: get("GITLAB_HOST"),
        GITLAB_CLIENT_ID: get("GITLAB_CLIENT_ID"),
        GITLAB_CLIENT_SECRET: get("GITLAB_CLIENT_SECRET"),
      });
    } catch (err) {
      console.error(err);
      setToast({
        type: TOAST_TYPE.ERROR,
        title: "Error",
        message: "Failed to save integration settings. Please try again.",
      });
    }
  };

  const handleGoBack = (e: React.MouseEvent<HTMLAnchorElement, MouseEvent>) => {
    if (isDirty) {
      e.preventDefault();
      setIsDiscardChangesModalOpen(true);
    }
  };

  return (
    <>
      <ConfirmDiscardModal
        isOpen={isDiscardChangesModalOpen}
        onDiscardHref="/integrations"
        handleClose={() => setIsDiscardChangesModalOpen(false)}
      />

      <div className="flex flex-col gap-8">
        {/* ── GitHub App ── */}
        <div className="flex flex-col gap-4 border-b border-subtle pb-8">
          {/* Header with toggle */}
          <div className="flex items-start justify-between gap-4">
            <div className="flex flex-col gap-1">
              <div className="text-lg font-medium">GitHub App</div>
              <p className="text-sm text-secondary">
                Required for the workspace Integrations panel. Create your GitHub App at{" "}
                <a
                  href="https://github.com/settings/apps/new"
                  target="_blank"
                  rel="noreferrer"
                  className="text-accent-primary hover:underline"
                >
                  github.com/settings/apps/new
                </a>{" "}
                and set the <strong>Setup URL (Callback)</strong> to the value shown below.
              </p>
            </div>
            <div className="flex shrink-0 items-center gap-2 pt-1">
              <span className="text-sm text-secondary">{isGithubEnabled ? "Enabled" : "Disabled"}</span>
              <ToggleSwitch
                value={Boolean(isGithubEnabled)}
                onChange={() => handleToggle("IS_GITHUB_ENABLED", isGithubEnabled)}
                size="sm"
              />
            </div>
          </div>

          {/* Fields */}
          <div className={isGithubEnabled === false ? "opacity-50" : undefined}>
            {/* Standard fields: App name, App ID, Private key */}
            {githubFields
              .filter((f) => f.key !== "GITHUB_WEBHOOK_SECRET")
              .map((field) => (
                <div key={field.key} className="mb-4 last:mb-0">
                  <ControllerInput
                    control={control}
                    type={field.type}
                    name={field.key}
                    label={field.label}
                    description={field.description}
                    placeholder={field.placeholder}
                    error={Boolean(errors[field.key as keyof typeof errors])}
                    required={isGithubEnabled ? field.required : false}
                    disabled={!isGithubEnabled}
                  />
                </div>
              ))}

            {/* Webhook secret with Generate button */}
            <div className="mb-4">
              <div className="flex items-end gap-2">
                <div className="flex-1">
                  <ControllerInput
                    control={control}
                    type="password"
                    name="GITHUB_WEBHOOK_SECRET"
                    label="Webhook secret"
                    description="Used to verify webhook payloads from GitHub. Generate one or set your own."
                    placeholder="xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
                    error={Boolean(errors.GITHUB_WEBHOOK_SECRET)}
                    required={false}
                    disabled={!isGithubEnabled}
                  />
                </div>
                <Button
                  variant="neutral-primary"
                  size="sm"
                  onClick={generateWebhookSecret}
                  type="button"
                  className="mb-0.5 shrink-0"
                  disabled={!isGithubEnabled}
                >
                  Generate
                </Button>
              </div>
            </div>
          </div>

          {/* Setup URLs info panel (outside the toggle opacity — always visible) */}
          <div className="flex flex-col gap-3 rounded-md border border-subtle bg-surface-2 p-4">
            <p className="text-sm font-medium">Setup URLs</p>

            <div className="flex flex-col gap-1">
              <p className="text-xs text-secondary">GitHub App Setup URL (Callback)</p>
              <div className="flex items-center gap-2">
                <code className="bg-surface-3 text-xs font-mono flex-1 rounded px-3 py-1.5 break-all">
                  {origin}/api/github/callback/
                </code>
                <button
                  type="button"
                  onClick={() => navigator.clipboard.writeText(`${origin}/api/github/callback/`)}
                  className="hover:bg-surface-3 shrink-0 rounded p-1.5 text-secondary hover:text-primary"
                  title="Copy"
                >
                  <Copy size={14} />
                </button>
              </div>
            </div>

            <div className="flex flex-col gap-1">
              <p className="text-xs text-secondary">Webhook URL</p>
              <div className="flex items-center gap-2">
                <code className="bg-surface-3 text-xs font-mono flex-1 rounded px-3 py-1.5 break-all">
                  {origin}/api/github-webhook/
                </code>
                <button
                  type="button"
                  onClick={() => navigator.clipboard.writeText(`${origin}/api/github-webhook/`)}
                  className="hover:bg-surface-3 shrink-0 rounded p-1.5 text-secondary hover:text-primary"
                  title="Copy"
                >
                  <Copy size={14} />
                </button>
              </div>
            </div>
          </div>
        </div>

        <Section
          title="GitLab"
          description={
            <>
              Required for GitLab integration. Create an OAuth application at{" "}
              <a
                href="https://gitlab.com/-/profile/applications"
                target="_blank"
                rel="noreferrer"
                className="text-accent-primary hover:underline"
              >
                gitlab.com/-/profile/applications
              </a>{" "}
              with redirect URI: <CodeBlock darkerShade>{origin}/auth/gitlab/callback</CodeBlock>.
            </>
          }
          fields={gitlabFields}
          control={control}
          errors={errors}
          toggleKey="IS_GITLAB_ENABLED"
          enabled={isGitlabEnabled}
          onToggle={() => handleToggle("IS_GITLAB_ENABLED", isGitlabEnabled)}
        />

        <Section
          title="Slack"
          description={
            <>
              Required for Slack integration. Create a Slack App at{" "}
              <a
                href="https://api.slack.com/apps"
                target="_blank"
                rel="noreferrer"
                className="text-accent-primary hover:underline"
              >
                api.slack.com/apps
              </a>
              .
            </>
          }
          fields={slackFields}
          control={control}
          errors={errors}
          toggleKey="IS_SLACK_ENABLED"
          enabled={isSlackEnabled}
          onToggle={() => handleToggle("IS_SLACK_ENABLED", isSlackEnabled)}
        />

        {/* Actions */}
        <div className="flex items-center gap-4 pt-2">
          <Button
            variant="primary"
            size="lg"
            onClick={(e) => void handleSubmit(onSubmit)(e)}
            loading={isSubmitting}
            disabled={!isDirty}
          >
            {isSubmitting ? "Saving…" : "Save changes"}
          </Button>
          <Link href="/general" className={getButtonStyling("secondary", "lg")} onClick={handleGoBack}>
            Go back
          </Link>
        </div>
      </div>
    </>
  );
});
