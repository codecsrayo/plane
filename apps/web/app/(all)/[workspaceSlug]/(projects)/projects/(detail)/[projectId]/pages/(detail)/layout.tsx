/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

// component
import { Outlet } from "react-router";
import useSWR from "swr";
import { AppHeader } from "@/components/core/app-header";
import { ContentWrapper } from "@/components/core/content-wrapper";
import { useProject } from "@/hooks/store/use-project";
// plane web hooks
import { EPageStoreType, usePageStore } from "@/plane-web/hooks/store";
// local components
import type { Route } from "./+types/layout";
import { PageDetailsHeader } from "./header";

export default function ProjectPageDetailsLayout({ params }: Route.ComponentProps) {
  const { workspaceSlug, projectId } = params;
  const { currentProjectDetails } = useProject();
  const { fetchPagesList } = usePageStore(EPageStoreType.PROJECT);
  // fetching pages list
  useSWR(currentProjectDetails?.page_view === false ? null : `PROJECT_PAGES_${projectId}`, () =>
    fetchPagesList(workspaceSlug, projectId)
  );

  const shouldShowHeader = currentProjectDetails?.page_view !== false;

  return (
    <>
      {shouldShowHeader && <AppHeader header={<PageDetailsHeader />} />}
      <ContentWrapper>
        <Outlet />
      </ContentWrapper>
    </>
  );
}
