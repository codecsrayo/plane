/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useState, useEffect } from "react";
import { observer } from "mobx-react";
import { useParams } from "react-router";
import useSWR, { mutate } from "swr";
// types
import type { IWorkspaceIntegration, ISlackIntegration } from "@plane/types";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
// ui
import { Loader } from "@plane/ui";
// fetch-keys
import { SLACK_CHANNEL_INFO } from "@/constants/fetch-keys";
// hooks
import { useInstance } from "@/hooks/store/use-instance";
import useIntegrationPopup from "@/hooks/use-integration-popup";
// services
import { AppInstallationService } from "@/services/app_installation.service";

type Props = {
  integration: IWorkspaceIntegration;
};

const appInstallationService = new AppInstallationService();

export const SelectChannel = observer(function SelectChannel({ integration }: Props) {
  // store hooks
  const { config } = useInstance();
  // states
  const [slackChannelAvailabilityToggle, setSlackChannelAvailabilityToggle] = useState<boolean>(false);
  const [slackChannel, setSlackChannel] = useState<ISlackIntegration | null>(null);

  const { workspaceSlug, projectId } = useParams();

  const { startAuth } = useIntegrationPopup({
    provider: "slackChannel",
    stateParams: integration.id,
    // github_app_name: instance?.config?.github_client_id || "",
    slack_client_id: config?.slack_client_id || "",
  });

  const swrKey = workspaceSlug && projectId && integration.id ? SLACK_CHANNEL_INFO(workspaceSlug, projectId) : null;

  const { data: projectIntegration } = useSWR<ISlackIntegration[]>(swrKey, () =>
    appInstallationService.getSlackChannelDetail(workspaceSlug as string, projectId as string, integration.id)
  );

  useEffect(() => {
    if (projectId && projectIntegration) {
      const projectSlackIntegrationCheck: ISlackIntegration | undefined = projectIntegration.find(
        (_slack: ISlackIntegration) => _slack.project === projectId
      );
      if (projectSlackIntegrationCheck) {
        setSlackChannel(() => projectSlackIntegrationCheck);
        setSlackChannelAvailabilityToggle(true);
        return;
      }

      setSlackChannel(null);
      setSlackChannelAvailabilityToggle(false);
    }
  }, [projectIntegration, projectId]);

  const handleDelete = async () => {
    if (!workspaceSlug || !projectId || !swrKey || !slackChannel?.id) return;

    try {
      await mutate(
        swrKey,
        async (currentData: ISlackIntegration[] = []) => {
          await appInstallationService.removeSlackChannel(workspaceSlug, projectId, integration.id, slackChannel.id);
          return currentData.filter((channel) => channel.id !== slackChannel.id);
        },
        {
          optimisticData: (currentData: ISlackIntegration[] = []) =>
            currentData.filter((channel) => channel.id !== slackChannel.id),
          rollbackOnError: true,
          revalidate: false,
        }
      );
      setSlackChannelAvailabilityToggle(false);
      setSlackChannel(null);
      setToast({ type: TOAST_TYPE.SUCCESS, title: "Slack channel disconnected" });
    } catch {
      setToast({ type: TOAST_TYPE.ERROR, title: "Failed to disconnect Slack channel" });
    }
  };

  return (
    <>
      {projectIntegration ? (
        <button
          type="button"
          className="bg-gray-700 relative inline-flex h-4 w-6 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none"
          role="switch"
          aria-checked={slackChannelAvailabilityToggle}
          onClick={() => {
            if (slackChannelAvailabilityToggle) {
              handleDelete();
            } else {
              startAuth();
            }
          }}
        >
          <span
            aria-hidden="true"
            className={`shadow inline-block size-2 transform self-center rounded-full bg-white ring-0 transition duration-200 ease-in-out ${
              slackChannelAvailabilityToggle ? "translate-x-3" : "translate-x-0"
            }`}
          />
        </button>
      ) : (
        <Loader>
          <Loader.Item height="35px" width="150px" />
        </Loader>
      )}
    </>
  );
});
