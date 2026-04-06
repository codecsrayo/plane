/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useState } from "react";
import Link from "next/link";
import { useForm } from "react-hook-form";
// plane internal packages
import { Button, getButtonStyling } from "@plane/propel/button";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
import type { IFormattedInstanceConfiguration, TInstanceIntegrationConfigurationKeys } from "@plane/types";
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

export function InstanceIntegrationsConfigForm({ config }: Props) {
  const [isDiscardChangesModalOpen, setIsDiscardChangesModalOpen] = useState(false);
  const { updateInstanceConfigurations } = useInstance();

  const {
    handleSubmit,
    control,
    reset,
    formState: { errors, isDirty, isSubmitting },
  } = useForm<IntegrationConfigFormValues>({
    defaultValues: {
      GITHUB_APP_NAME: config["GITHUB_APP_NAME"] ?? "",
      SLACK_CLIENT_ID: config["SLACK_CLIENT_ID"] ?? "",
      SLACK_CLIENT_SECRET: config["SLACK_CLIENT_SECRET"] ?? "",
    },
  });

  const GITHUB_FIELDS: TControllerInputFormField[] = [
    {
      key: "GITHUB_APP_NAME",
      type: "text",
      label: "GitHub App name",
      description: (
        <>
          The slug of your{" "}
          <a
            tabIndex={-1}
            href="https://github.com/settings/apps"
            target="_blank"
            rel="noreferrer"
            className="text-accent-primary hover:underline"
          >
            GitHub App
          </a>
          . It appears in the install URL:{" "}
          <CodeBlock darkerShade>github.com/apps/&lt;THIS-NAME&gt;/installations/new</CodeBlock>
        </>
      ),
      placeholder: "your-github-app-name",
      error: Boolean(errors.GITHUB_APP_NAME),
      required: true,
    },
  ];

  const SLACK_FIELDS: TControllerInputFormField[] = [
    {
      key: "SLACK_CLIENT_ID",
      type: "text",
      label: "Client ID",
      description: (
        <>
          Found in your{" "}
          <a
            tabIndex={-1}
            href="https://api.slack.com/apps"
            target="_blank"
            rel="noreferrer"
            className="text-accent-primary hover:underline"
          >
            Slack App settings
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

  const onSubmit = async (formData: IntegrationConfigFormValues) => {
    try {
      const response = await updateInstanceConfigurations({ ...formData });
      setToast({
        type: TOAST_TYPE.SUCCESS,
        title: "Saved!",
        message: "Integration settings updated successfully.",
      });
      reset({
        GITHUB_APP_NAME: response.find((i) => i.key === "GITHUB_APP_NAME")?.value ?? "",
        SLACK_CLIENT_ID: response.find((i) => i.key === "SLACK_CLIENT_ID")?.value ?? "",
        SLACK_CLIENT_SECRET: response.find((i) => i.key === "SLACK_CLIENT_SECRET")?.value ?? "",
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
      <div className="flex flex-col gap-10">
        {/* GitHub App section */}
        <div className="flex flex-col gap-4">
          <div className="text-18 font-medium">GitHub App</div>
          <p className="text-sm text-secondary">
            Required to enable the Install button in the workspace Integrations panel. Create your GitHub App at{" "}
            <a
              href="https://github.com/settings/apps/new"
              target="_blank"
              rel="noreferrer"
              className="text-accent-primary hover:underline"
            >
              github.com/settings/apps/new
            </a>{" "}
            and set the callback URL to{" "}
            <CodeBlock darkerShade>
              {typeof window !== "undefined" ? window.location.origin : ""}/auth/github/callback
            </CodeBlock>
          </p>
          {GITHUB_FIELDS.map((field) => (
            <ControllerInput
              key={field.key}
              control={control}
              type={field.type}
              name={field.key}
              label={field.label}
              description={field.description}
              placeholder={field.placeholder}
              error={field.error}
              required={field.required}
            />
          ))}
        </div>

        {/* Slack section */}
        <div className="flex flex-col gap-4">
          <div className="text-18 font-medium">Slack</div>
          <p className="text-sm text-secondary">
            Required to enable Slack integration in the workspace Integrations panel. Create your Slack App at{" "}
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
          {SLACK_FIELDS.map((field) => (
            <ControllerInput
              key={field.key}
              control={control}
              type={field.type}
              name={field.key}
              label={field.label}
              description={field.description}
              placeholder={field.placeholder}
              error={field.error}
              required={field.required}
            />
          ))}
        </div>

        {/* Actions */}
        <div className="flex items-center gap-4">
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
}
