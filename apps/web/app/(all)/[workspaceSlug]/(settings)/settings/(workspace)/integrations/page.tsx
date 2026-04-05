/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { observer } from "mobx-react";
import useSWR from "swr";
// components
import { EUserPermissions, EUserPermissionsLevel } from "@plane/constants";
import { NotAuthorizedView } from "@/components/auth-screens/not-authorized-view";
import { PageHead } from "@/components/core/page-title";
import { SingleIntegrationCard } from "@/components/integration/single-integration-card";
import { IntegrationAndImportExportBanner } from "@/components/ui/integration-and-import-export-banner";
import { IntegrationsSettingsLoader } from "@/components/ui/loader/settings/integration";
// constants
import { APP_INTEGRATIONS } from "@/constants/fetch-keys";
// hooks
import { useWorkspace } from "@/hooks/store/use-workspace";
import { useUserPermissions } from "@/hooks/store/user";
// services
import { IntegrationService } from "@/services/integrations";

const integrationService = new IntegrationService();

function WorkspaceIntegrationsPage() {
  // store hooks
  const { currentWorkspace } = useWorkspace();
  const { allowPermissions } = useUserPermissions();

  // derived values
  const isAdmin = allowPermissions([EUserPermissions.ADMIN], EUserPermissionsLevel.WORKSPACE);
  const pageTitle = currentWorkspace?.name ? `${currentWorkspace.name} - Integrations` : undefined;

  // fetch integrations — key is null when not admin so SWR skips the call
  const {
    data: appIntegrations,
    isLoading: isIntegrationsLoading,
    error: integrationsError,
  } = useSWR(
    isAdmin ? APP_INTEGRATIONS : null,
    () => (isAdmin ? integrationService.getAppIntegrationsList() : null),
    { shouldRetryOnError: false }
  );

  if (!isAdmin) return <NotAuthorizedView section="settings" className="h-auto" />;

  // Determine render state
  const isLoading = isIntegrationsLoading || (!appIntegrations && !integrationsError);
  const hasIntegrations = Array.isArray(appIntegrations) && appIntegrations.length > 0;

  return (
    <>
      <PageHead title={pageTitle} />
      <section className="w-full overflow-y-auto">
        <IntegrationAndImportExportBanner bannerName="Integrations" />
        <div>
          {isLoading ? (
            <IntegrationsSettingsLoader />
          ) : integrationsError ? (
            <p className="px-4 py-6 text-sm text-red-500">
              Failed to load integrations. Please refresh the page and try again.
            </p>
          ) : hasIntegrations ? (
            appIntegrations.map((integration) => (
              <SingleIntegrationCard key={integration.id} integration={integration} />
            ))
          ) : (
            <p className="px-4 py-6 text-sm text-secondary">No integrations are available at the moment.</p>
          )}
        </div>
      </section>
    </>
  );
}

export default observer(WorkspaceIntegrationsPage);
