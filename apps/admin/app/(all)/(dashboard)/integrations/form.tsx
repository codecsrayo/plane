/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import { observer } from "mobx-react";
import { Check, Copy, KeyRound, Monitor } from "lucide-react";
import { Controller, useForm } from "react-hook-form";
// plane internal packages
import { Button, getButtonStyling } from "@plane/propel/button";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
import type {
  IFormattedInstanceConfiguration,
  TInstanceAuthenticationMethodKeys,
  TInstanceIntegrationConfigurationKeys,
} from "@plane/types";
import { TextArea, ToggleSwitch } from "@plane/ui";
// components
import { CodeBlock } from "@/components/common/code-block";
import { ConfirmDiscardModal } from "@/components/common/confirm-discard-modal";
import type { TControllerInputFormField } from "@/components/common/controller-input";
import { ControllerInput } from "@/components/common/controller-input";
import { CopyField } from "@/components/common/copy-field";
// hooks
import { useInstance } from "@/hooks/store";

type Props = {
  config: IFormattedInstanceConfiguration;
};

type IntegrationConfigFormValues = Record<TInstanceIntegrationConfigurationKeys, string>;

const PEM_HEADER = "-----BEGIN";
const decodeBase64 = (value: string) => globalThis.atob(value);
const encodeBase64 = (value: string) => globalThis.btoa(value);

const normalizePemBlock = (value: string) => {
  const trimmedValue = value.trim();
  if (!trimmedValue.startsWith(PEM_HEADER)) return trimmedValue;

  const lines = trimmedValue
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);

  if (lines.length < 3) return trimmedValue;

  const beginLine = lines[0];
  const endLine = lines.at(-1);
  if (!endLine?.startsWith("-----END")) return trimmedValue;

  const body = lines.slice(1, -1).join("");
  return `${beginLine}\n${body}\n${endLine}`;
};

const decodePrivateKeyForDisplay = (value: string) => {
  if (!value) return "";

  const trimmedValue = value.trim();
  if (trimmedValue.startsWith(PEM_HEADER)) return normalizePemBlock(trimmedValue);

  try {
    const decodedValue = decodeBase64(trimmedValue).trim();
    return decodedValue.startsWith(PEM_HEADER) ? normalizePemBlock(decodedValue) : value;
  } catch {
    return value;
  }
};

const encodePrivateKeyForStorage = (value: string) => {
  const trimmedValue = value.trim();
  if (!trimmedValue) return "";

  if (trimmedValue.startsWith(PEM_HEADER)) return encodeBase64(normalizePemBlock(trimmedValue));

  try {
    const decodedValue = decodeBase64(trimmedValue).trim();
    return decodedValue.startsWith(PEM_HEADER) ? trimmedValue : encodeBase64(trimmedValue);
  } catch {
    return encodeBase64(trimmedValue);
  }
};

// ─── Main form ────────────────────────────────────────────────────────────────
export const InstanceIntegrationsConfigForm = observer(function InstanceIntegrationsConfigForm({ config }: Props) {
  const [isDiscardChangesModalOpen, setIsDiscardChangesModalOpen] = useState(false);
  const [isWebhookSecretCopied, setIsWebhookSecretCopied] = useState(false);
  const [isGithubPrivateKeySaved, setIsGithubPrivateKeySaved] = useState(Boolean(config["GITHUB_APP_PRIVATE_KEY"]));
  const [githubPrivateKeyLabel, setGithubPrivateKeyLabel] = useState(
    config["GITHUB_APP_PRIVATE_KEY"] ? "GitHub App private key" : ""
  );
  const webhookSecretCopyTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const githubPrivateKeyFileInputRef = useRef<HTMLInputElement | null>(null);
  const { formattedConfig, updateInstanceConfigurations } = useInstance();

  useEffect(
    () => () => {
      if (webhookSecretCopyTimeoutRef.current) clearTimeout(webhookSecretCopyTimeoutRef.current);
    },
    []
  );

  // ── Toggle helpers ─────────────────────────────────────────────────────────
  const isGithubEnabled = Boolean(parseInt(formattedConfig?.IS_GITHUB_INTEGRATION_ENABLED ?? "0"));
  const isGitlabEnabled = Boolean(parseInt(formattedConfig?.IS_GITLAB_INTEGRATION_ENABLED ?? "0"));
  const isSlackEnabled = Boolean(parseInt(formattedConfig?.IS_SLACK_ENABLED ?? "0"));

  const handleToggle = (key: TInstanceAuthenticationMethodKeys, current: boolean) => {
    updateInstanceConfigurations({ [key]: current ? "0" : "1" } as Record<string, string>).catch(console.error);
  };

  const origin = typeof window !== "undefined" ? window.location.origin : "";

  const {
    handleSubmit,
    control,
    getValues,
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

  const copyWebhookSecret = async () => {
    const secret = getValues("GITHUB_WEBHOOK_SECRET");
    if (!secret) return;

    try {
      await navigator.clipboard.writeText(secret);
      setIsWebhookSecretCopied(true);
      if (webhookSecretCopyTimeoutRef.current) clearTimeout(webhookSecretCopyTimeoutRef.current);
      webhookSecretCopyTimeoutRef.current = setTimeout(() => {
        setIsWebhookSecretCopied(false);
        webhookSecretCopyTimeoutRef.current = null;
      }, 2000);
    } catch {
      setIsWebhookSecretCopied(false);
    }
  };

  const handleGithubPrivateKeyUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    try {
      const pem = await file.text();
      const pemBase64 = encodeBase64(pem);
      setValue("GITHUB_APP_PRIVATE_KEY", pemBase64, { shouldDirty: true, shouldValidate: true });
      setIsGithubPrivateKeySaved(false);
      setGithubPrivateKeyLabel(file.name);
      setToast({
        type: TOAST_TYPE.SUCCESS,
        title: "Private key loaded",
        message: `${file.name} was converted to base64 and loaded into the form.`,
      });
    } catch {
      setToast({
        type: TOAST_TYPE.ERROR,
        title: "Invalid private key",
        message: "Failed to read the .pem file. Please try again.",
      });
    } finally {
      event.target.value = "";
    }
  };

  const handleGithubPrivateKeySave = () => {
    const privateKeyValue = getValues("GITHUB_APP_PRIVATE_KEY");
    if (!privateKeyValue.trim()) {
      setToast({
        type: TOAST_TYPE.ERROR,
        title: "Private key required",
        message: "Upload or paste a private key before saving it.",
      });
      return;
    }

    setIsGithubPrivateKeySaved(true);
    if (!githubPrivateKeyLabel) setGithubPrivateKeyLabel("GitHub App private key");
  };

  const handleGithubPrivateKeyDelete = () => {
    setValue("GITHUB_APP_PRIVATE_KEY", "", { shouldDirty: true, shouldValidate: true });
    setIsGithubPrivateKeySaved(false);
    setGithubPrivateKeyLabel("");
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

      <div className="flex flex-col gap-12">
        {/* ── GitHub App ── */}
        <div className="flex flex-col gap-6 border-b border-subtle pb-10">
          {/* Section header with toggle */}
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
                </a>
                .
              </p>
            </div>
            <div className="flex shrink-0 items-center gap-2 pt-1">
              <span className="text-sm text-secondary">{isGithubEnabled ? "Enabled" : "Disabled"}</span>
              <ToggleSwitch
                value={Boolean(isGithubEnabled)}
                onChange={() => handleToggle("IS_GITHUB_INTEGRATION_ENABLED", isGithubEnabled)}
                size="sm"
              />
            </div>
          </div>

          {/* Two-column body */}
          <div className="grid w-full grid-cols-2 gap-x-12 gap-y-8">
            {/* Left — GitHub-provided details for Plane */}
            <div className="col-span-2 flex flex-col gap-y-4 md:col-span-1">
              <div className="pt-2.5 text-18 font-medium">GitHub-provided details for Plane</div>
              <div className={isGithubEnabled === false ? "flex flex-col gap-y-4 opacity-50" : "flex flex-col gap-y-4"}>
                {githubFields
                  .filter((f) => f.key !== "GITHUB_WEBHOOK_SECRET")
                  .map((field) =>
                    field.key === "GITHUB_APP_PRIVATE_KEY" ? (
                      <div key={field.key} className="flex flex-col gap-2">
                        {isGithubPrivateKeySaved ? (
                          <div className="flex items-center justify-between gap-4 rounded-lg border border-subtle bg-layer-1 px-5 py-4">
                            <div className="flex items-center gap-4">
                              <div className="flex h-10 w-10 items-center justify-center rounded-full bg-layer-2">
                                <KeyRound className="h-5 w-5" />
                              </div>
                              <div className="flex flex-col">
                                <p className="text-base font-medium">
                                  {githubPrivateKeyLabel || "GitHub App private key"}
                                </p>
                                <p className="font-mono text-xs text-secondary">RSA private key configured</p>
                                <p className="text-sm text-secondary">
                                  Saved locally in this form. Use Save changes to persist it.
                                </p>
                              </div>
                            </div>
                            <Button
                              type="button"
                              variant="secondary"
                              size="sm"
                              onClick={handleGithubPrivateKeyDelete}
                              disabled={!isGithubEnabled}
                              className="text-destructive"
                            >
                              Delete
                            </Button>
                          </div>
                        ) : (
                          <>
                            <div className="flex flex-col gap-1">
                              <h4 className="text-13 text-tertiary">Private key</h4>
                              <Controller
                                control={control}
                                name={field.key}
                                rules={{ required: isGithubEnabled ? "Private key is required." : false }}
                                render={({ field: { value, onChange, ref } }) => (
                                  <TextArea
                                    id={field.key}
                                    name={field.key}
                                    ref={ref}
                                    value={decodePrivateKeyForDisplay(value)}
                                    onChange={(e) => {
                                      setIsGithubPrivateKeySaved(false);
                                      onChange(encodePrivateKeyForStorage(e.target.value));
                                    }}
                                    hasError={Boolean(errors[field.key as keyof typeof errors])}
                                    placeholder={
                                      "-----BEGIN RSA PRIVATE KEY-----\nMIIE...\n-----END RSA PRIVATE KEY-----"
                                    }
                                    disabled={!isGithubEnabled}
                                    rows={8}
                                    className="font-mono text-sm h-64 min-h-64 w-full resize-y rounded-md"
                                  />
                                )}
                              />
                              <p className="pt-0.5 text-11 text-tertiary">
                                <>
                                  Upload your GitHub App <CodeBlock darkerShade>.pem</CodeBlock> file or paste the
                                  base64 value directly. The form stores the value in base64 for backend compatibility.
                                </>
                              </p>
                            </div>
                            <div className="flex items-center gap-2">
                              <input
                                ref={githubPrivateKeyFileInputRef}
                                type="file"
                                accept=".pem"
                                className="hidden"
                                onChange={(e) => void handleGithubPrivateKeyUpload(e)}
                                disabled={!isGithubEnabled}
                              />
                              <Button
                                type="button"
                                variant="secondary"
                                size="sm"
                                onClick={() => githubPrivateKeyFileInputRef.current?.click()}
                                disabled={!isGithubEnabled}
                              >
                                Upload .pem
                              </Button>
                              <Button
                                type="button"
                                variant="primary"
                                size="sm"
                                onClick={handleGithubPrivateKeySave}
                                disabled={!isGithubEnabled}
                              >
                                Guardar
                              </Button>
                              <p className="text-11 text-tertiary">
                                GitHub generates a PEM private key. Plane converts it to base64 before saving.
                              </p>
                            </div>
                          </>
                        )}
                      </div>
                    ) : (
                      <ControllerInput
                        key={field.key}
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
                    )
                  )}
              </div>
            </div>

            {/* Right — Plane-provided details for GitHub */}
            <div className="col-span-2 flex flex-col gap-y-6 md:col-span-1">
              <div className="pt-2 text-18 font-medium">Plane-provided details for GitHub</div>
              <div className="flex flex-col gap-y-4">
                {/* Callback URL */}
                <div className="flex flex-col gap-y-4 rounded-lg bg-layer-1 px-6 py-4">
                  <CopyField
                    label="Setup URL (Callback)"
                    url={`${origin}/api/github/callback/`}
                    description={
                      <p>
                        Paste this into the <CodeBlock darkerShade>Setup URL (optional)</CodeBlock> field when creating
                        your GitHub App.
                      </p>
                    }
                  />
                </div>
                {/* Webhook URL */}
                <div className="flex flex-col overflow-hidden rounded-lg">
                  <div className="flex items-center gap-x-3 bg-layer-3 px-6 py-3 text-11 font-medium text-secondary uppercase">
                    <Monitor className="h-3 w-3" />
                    Webhooks
                  </div>
                  <div className="flex flex-col gap-y-4 bg-layer-1 px-6 py-4">
                    <CopyField
                      label="Webhook URL"
                      url={`${origin}/api/github-webhook/`}
                      description={
                        <p>
                          Paste this into the <CodeBlock darkerShade>Webhook URL</CodeBlock> field in your GitHub App
                          settings.
                        </p>
                      }
                    />
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
                          rightContent={
                            <button
                              type="button"
                              tabIndex={-1}
                              className="flex items-center justify-center"
                              onClick={() => void copyWebhookSecret()}
                              disabled={!isGithubEnabled || !getValues("GITHUB_WEBHOOK_SECRET")}
                            >
                              {isWebhookSecretCopied ? (
                                <Check className="h-4 w-4 text-success-primary" />
                              ) : (
                                <Copy className="h-4 w-4" />
                              )}
                            </button>
                          }
                        />
                      </div>
                      <Button
                        variant="secondary"
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
              </div>
            </div>
          </div>
        </div>

        {/* ── GitLab ── */}
        <div className="flex flex-col gap-6 border-b border-subtle pb-10">
          {/* Section header with toggle */}
          <div className="flex items-start justify-between gap-4">
            <div className="flex flex-col gap-1">
              <div className="text-lg font-medium">GitLab</div>
              <p className="text-sm text-secondary">
                Required for GitLab integration. Create an OAuth application at{" "}
                <a
                  href="https://gitlab.com/-/profile/applications"
                  target="_blank"
                  rel="noreferrer"
                  className="text-accent-primary hover:underline"
                >
                  gitlab.com/-/profile/applications
                </a>
                .
              </p>
            </div>
            <div className="flex shrink-0 items-center gap-2 pt-1">
              <span className="text-sm text-secondary">{isGitlabEnabled ? "Enabled" : "Disabled"}</span>
              <ToggleSwitch
                value={Boolean(isGitlabEnabled)}
                onChange={() => handleToggle("IS_GITLAB_INTEGRATION_ENABLED", isGitlabEnabled)}
                size="sm"
              />
            </div>
          </div>

          {/* Two-column body */}
          <div className="grid w-full grid-cols-2 gap-x-12 gap-y-8">
            {/* Left — GitLab-provided details for Plane */}
            <div className="col-span-2 flex flex-col gap-y-4 md:col-span-1">
              <div className="pt-2.5 text-18 font-medium">GitLab-provided details for Plane</div>
              <div className={isGitlabEnabled === false ? "flex flex-col gap-y-4 opacity-50" : "flex flex-col gap-y-4"}>
                {gitlabFields.map((field) => (
                  <ControllerInput
                    key={field.key}
                    control={control}
                    type={field.type}
                    name={field.key}
                    label={field.label}
                    description={field.description}
                    placeholder={field.placeholder}
                    error={Boolean(errors[field.key as keyof typeof errors])}
                    required={isGitlabEnabled ? field.required : false}
                    disabled={!isGitlabEnabled}
                  />
                ))}
              </div>
            </div>

            {/* Right — Plane-provided details for GitLab */}
            <div className="col-span-2 flex flex-col gap-y-6 md:col-span-1">
              <div className="pt-2 text-18 font-medium">Plane-provided details for GitLab</div>
              <div className="flex flex-col gap-y-4">
                <div className="flex flex-col overflow-hidden rounded-lg">
                  <div className="flex items-center gap-x-3 bg-layer-3 px-6 py-3 text-11 font-medium text-secondary uppercase">
                    <Monitor className="h-3 w-3" />
                    OAuth
                  </div>
                  <div className="flex flex-col gap-y-4 bg-layer-1 px-6 py-4">
                    <CopyField
                      label="Redirect URI"
                      url={`${origin}/auth/gitlab/callback`}
                      description={
                        <p>
                          Paste this into the <CodeBlock darkerShade>Redirect URI</CodeBlock> field when creating your
                          GitLab OAuth application.
                        </p>
                      }
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* ── Slack ── */}
        <div className="flex flex-col gap-6 border-b border-subtle pb-10 last:border-none">
          {/* Section header with toggle */}
          <div className="flex items-start justify-between gap-4">
            <div className="flex flex-col gap-1">
              <div className="text-lg font-medium">Slack</div>
              <p className="text-sm text-secondary">
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
              </p>
            </div>
            <div className="flex shrink-0 items-center gap-2 pt-1">
              <span className="text-sm text-secondary">{isSlackEnabled ? "Enabled" : "Disabled"}</span>
              <ToggleSwitch
                value={Boolean(isSlackEnabled)}
                onChange={() => handleToggle("IS_SLACK_ENABLED", isSlackEnabled)}
                size="sm"
              />
            </div>
          </div>

          {/* Two-column body */}
          <div className="grid w-full grid-cols-2 gap-x-12 gap-y-8">
            {/* Left — Slack-provided details for Plane */}
            <div className="col-span-2 flex flex-col gap-y-4 md:col-span-1">
              <div className="pt-2.5 text-18 font-medium">Slack-provided details for Plane</div>
              <div className={isSlackEnabled === false ? "flex flex-col gap-y-4 opacity-50" : "flex flex-col gap-y-4"}>
                {slackFields.map((field) => (
                  <ControllerInput
                    key={field.key}
                    control={control}
                    type={field.type}
                    name={field.key}
                    label={field.label}
                    description={field.description}
                    placeholder={field.placeholder}
                    error={Boolean(errors[field.key as keyof typeof errors])}
                    required={isSlackEnabled ? field.required : false}
                    disabled={!isSlackEnabled}
                  />
                ))}
              </div>
            </div>

            {/* Right — Plane-provided details for Slack */}
            <div className="col-span-2 flex flex-col gap-y-6 md:col-span-1">
              <div className="pt-2 text-18 font-medium">Plane-provided details for Slack</div>
              <div className="flex flex-col gap-y-4">
                <div className="flex flex-col overflow-hidden rounded-lg">
                  <div className="flex items-center gap-x-3 bg-layer-3 px-6 py-3 text-11 font-medium text-secondary uppercase">
                    <Monitor className="h-3 w-3" />
                    OAuth
                  </div>
                  <div className="flex flex-col gap-y-4 bg-layer-1 px-6 py-4">
                    <CopyField
                      label="Redirect URL"
                      url={`${origin}/auth/slack/callback/`}
                      description={
                        <p>
                          Paste this into the <CodeBlock darkerShade>Redirect URL</CodeBlock> field under OAuth &
                          Permissions in your Slack App settings.
                        </p>
                      }
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

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
