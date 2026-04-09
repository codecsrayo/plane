/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useCallback, useEffect, useMemo } from "react";
import { observer } from "mobx-react";
import Link from "next/link";
import useSWR from "swr";
// plane types
import { EUserPermissionsLevel } from "@plane/constants";
import { getButtonStyling } from "@plane/propel/button";
import { useTranslation } from "@plane/i18n";
import type { TSearchEntityRequestPayload, TWebhookConnectionQueryParams } from "@plane/types";
import { EFileAssetType } from "@plane/types";
import { EUserProjectRoles } from "@plane/types";
import { useTheme } from "next-themes";
// plane ui
// plane utils
import { cn } from "@plane/utils";
// assets
import darkPagesAsset from "@/app/assets/empty-state/disabled-feature/pages-dark.webp?url";
import lightPagesAsset from "@/app/assets/empty-state/disabled-feature/pages-light.webp?url";
// components
import { LogoSpinner } from "@/components/common/logo-spinner";
import { PageHead } from "@/components/core/page-title";
import { DetailedEmptyState } from "@/components/empty-state/detailed-empty-state-root";
import { IssuePeekOverview } from "@/components/issues/peek-overview";
import type { TPageRootConfig, TPageRootHandlers } from "@/components/pages/editor/page-root";
import { PageRoot } from "@/components/pages/editor/page-root";
// hooks
import { useEditorConfig } from "@/hooks/editor";
import { useEditorAsset } from "@/hooks/store/use-editor-asset";
import { useProject } from "@/hooks/store/use-project";
import { useWorkspace } from "@/hooks/store/use-workspace";
import { useUserPermissions } from "@/hooks/store/user";
import { useAppRouter } from "@/hooks/use-app-router";
// plane web hooks
import { EPageStoreType, usePage, usePageStore } from "@/plane-web/hooks/store";
// plane web services
import { WorkspaceService } from "@/services/workspace.service";
// services
import { ProjectPageService, ProjectPageVersionService } from "@/services/page";
import type { Route } from "./+types/page";
const workspaceService = new WorkspaceService();
const projectPageService = new ProjectPageService();
const projectPageVersionService = new ProjectPageVersionService();

const storeType = EPageStoreType.PROJECT;

function PageDetailsPage({ params }: Route.ComponentProps) {
  // router
  const router = useAppRouter();
  const { workspaceSlug, projectId, pageId } = params;
  // theme/hooks
  const { resolvedTheme } = useTheme();
  const { t } = useTranslation();
  // store hooks
  const { createPage, fetchPageDetails } = usePageStore(storeType);
  const page = usePage({
    pageId,
    storeType,
  });
  const { currentProjectDetails } = useProject();
  const { getWorkspaceBySlug } = useWorkspace();
  const { uploadEditorAsset, duplicateEditorAsset } = useEditorAsset();
  const { allowPermissions } = useUserPermissions();
  // derived values
  const workspaceId = workspaceSlug ? (getWorkspaceBySlug(workspaceSlug)?.id ?? "") : "";
  const { canCurrentUserAccessPage, id, name, updateDescription } = page ?? {};
  const canPerformEmptyStateActions = allowPermissions([EUserProjectRoles.ADMIN], EUserPermissionsLevel.PROJECT);
  const resolvedPath = resolvedTheme === "light" ? lightPagesAsset : darkPagesAsset;
  // entity search handler
  const fetchEntityCallback = useCallback(
    async (payload: TSearchEntityRequestPayload) =>
      await workspaceService.searchEntity(workspaceSlug, {
        ...payload,
        project_id: projectId,
      }),
    [projectId, workspaceSlug]
  );
  // editor config
  const { getEditorFileHandlers } = useEditorConfig();
  // fetch page details
  const { error: pageDetailsError } = useSWR(
    currentProjectDetails?.page_view === false ? null : `PAGE_DETAILS_${pageId}`,
    () => fetchPageDetails(workspaceSlug, projectId, pageId),
    {
      revalidateIfStale: true,
      revalidateOnFocus: true,
      revalidateOnReconnect: true,
    }
  );
  // page root handlers
  const pageRootHandlers: TPageRootHandlers = useMemo(
    () => ({
      create: createPage,
      fetchAllVersions: async (targetPageId) =>
        await projectPageVersionService.fetchAllVersions(workspaceSlug, projectId, targetPageId),
      fetchDescriptionBinary: async () => {
        if (!id) return;
        return await projectPageService.fetchDescriptionBinary(workspaceSlug, projectId, id);
      },
      fetchEntity: fetchEntityCallback,
      fetchVersionDetails: async (targetPageId, versionId) =>
        await projectPageVersionService.fetchVersionById(workspaceSlug, projectId, targetPageId, versionId),
      restoreVersion: async (targetPageId, versionId) =>
        await projectPageVersionService.restoreVersion(workspaceSlug, projectId, targetPageId, versionId),
      getRedirectionLink: (targetPageId) => {
        if (targetPageId) {
          return `/${workspaceSlug}/projects/${projectId}/pages/${targetPageId}`;
        } else {
          return `/${workspaceSlug}/projects/${projectId}/pages`;
        }
      },
      updateDescription: updateDescription ?? (async () => {}),
    }),
    [createPage, fetchEntityCallback, id, updateDescription, workspaceSlug, projectId]
  );
  // page root config
  const pageRootConfig: TPageRootConfig = useMemo(
    () => ({
      fileHandler: getEditorFileHandlers({
        projectId,
        uploadFile: async (blockId, file) => {
          const { asset_id } = await uploadEditorAsset({
            blockId,
            data: {
              entity_identifier: id ?? "",
              entity_type: EFileAssetType.PAGE_DESCRIPTION,
            },
            file,
            projectId,
            workspaceSlug,
          });
          return asset_id;
        },
        duplicateFile: async (assetId: string) => {
          const { asset_id } = await duplicateEditorAsset({
            assetId,
            entityId: id,
            entityType: EFileAssetType.PAGE_DESCRIPTION,
            projectId,
            workspaceSlug,
          });
          return asset_id;
        },
        workspaceId,
        workspaceSlug,
      }),
    }),
    [getEditorFileHandlers, projectId, workspaceId, workspaceSlug, uploadEditorAsset, id, duplicateEditorAsset]
  );

  const webhookConnectionParams: TWebhookConnectionQueryParams = useMemo(
    () => ({
      documentType: "project_page",
      projectId,
      workspaceSlug,
    }),
    [projectId, workspaceSlug]
  );

  useEffect(() => {
    if (page?.deleted_at && page?.id) {
      router.push(pageRootHandlers.getRedirectionLink());
    }
  }, [page?.deleted_at, page?.id, router, pageRootHandlers]);

  if (currentProjectDetails?.page_view === false)
    return (
      <div className="flex h-full w-full items-center justify-center">
        <DetailedEmptyState
          title={t("disabled_project.empty_state.page.title")}
          description={t("disabled_project.empty_state.page.description")}
          assetPath={resolvedPath}
          primaryButton={{
            text: t("disabled_project.empty_state.page.primary_button.text"),
            onClick: () => {
              router.push(`/${workspaceSlug}/settings/projects/${projectId}/features`);
            },
            disabled: !canPerformEmptyStateActions,
          }}
        />
      </div>
    );

  if ((!page || !id) && !pageDetailsError)
    return (
      <div className="grid size-full place-items-center">
        <LogoSpinner />
      </div>
    );

  if (pageDetailsError || !canCurrentUserAccessPage)
    return (
      <div className="flex h-full w-full flex-col items-center justify-center">
        <h3 className="text-center text-16 font-semibold">Page not found</h3>
        <p className="mt-3 text-center text-13 text-secondary">
          The page you are trying to access doesn{"'"}t exist or you don{"'"}t have permission to view it.
        </p>
        <Link
          href={`/${workspaceSlug}/projects/${projectId}/pages`}
          className={cn(getButtonStyling("secondary", "base"), "mt-5")}
        >
          View other Pages
        </Link>
      </div>
    );

  if (!page) return null;

  return (
    <>
      <PageHead title={name} />
      <div className="flex h-full flex-col justify-between">
        <div className="relative flex h-full w-full flex-shrink-0 flex-col overflow-hidden">
          <PageRoot
            config={pageRootConfig}
            handlers={pageRootHandlers}
            storeType={storeType}
            page={page}
            webhookConnectionParams={webhookConnectionParams}
            workspaceSlug={workspaceSlug}
            projectId={projectId}
          />
          <IssuePeekOverview />
        </div>
      </div>
    </>
  );
}

export default observer(PageDetailsPage);
