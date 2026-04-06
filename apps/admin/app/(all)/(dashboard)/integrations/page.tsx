/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { observer } from "mobx-react";
import useSWR from "swr";
// plane internal packages
import { Loader } from "@plane/ui";
// components
import { PageWrapper } from "@/components/common/page-wrapper";
// hooks
import { useInstance } from "@/hooks/store";
// types
import type { Route } from "./+types/page";
// local
import { InstanceIntegrationsConfigForm } from "./form";

const InstanceIntegrationsPage = observer(function InstanceIntegrationsPage(_props: Route.ComponentProps) {
  const { fetchInstanceConfigurations, formattedConfig } = useInstance();

  useSWR("INSTANCE_CONFIGURATIONS", () => fetchInstanceConfigurations());

  return (
    <PageWrapper
      header={
        <div className="flex flex-col gap-1">
          <h3 className="text-xl font-medium">Integrations</h3>
          <p className="text-sm text-secondary">
            Configure third-party integrations available to all workspaces on this instance.
          </p>
        </div>
      }
    >
      {formattedConfig ? (
        <InstanceIntegrationsConfigForm config={formattedConfig} />
      ) : (
        <Loader className="space-y-8">
          <Loader.Item height="50px" width="40%" />
          <Loader.Item height="50px" />
          <Loader.Item height="50px" />
          <Loader.Item height="50px" />
          <Loader.Item height="50px" width="25%" />
        </Loader>
      )}
    </PageWrapper>
  );
});

export const meta: Route.MetaFunction = () => [{ title: "Integrations - God Mode" }];

export default InstanceIntegrationsPage;
