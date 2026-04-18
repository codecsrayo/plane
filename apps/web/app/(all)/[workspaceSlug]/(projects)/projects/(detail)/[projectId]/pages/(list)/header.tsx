/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useState } from "react";
import { observer } from "mobx-react";
import { useParams, useRouter, useSearchParams } from "next/navigation";
// constants
import { EPageAccess } from "@plane/constants";
// plane types
import { Button } from "@plane/propel/button";
import { PageIcon } from "@plane/propel/icons";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
import type { TPage } from "@plane/types";
// plane ui
import { Breadcrumbs, Header } from "@plane/ui";
// helpers
import { BreadcrumbLink } from "@/components/common/breadcrumb-link";
import { useProject } from "@/hooks/store/use-project";
// plane web imports
import { CommonProjectBreadcrumbs } from "@/plane-web/components/breadcrumbs/common";
import { EPageStoreType, usePageStore } from "@/plane-web/hooks/store";

export const PagesListHeader = observer(function PagesListHeader() {
  // states
  const [isCreatingPage, setIsCreatingPage] = useState(false);
  // router
  const router = useRouter();
  const { workspaceSlug, projectId } = useParams();
  const searchParams = useSearchParams();
  const pageType = searchParams.get("type");
  // store hooks
  const projectIdParam = projectId?.toString();
  const workspaceSlugParam = workspaceSlug?.toString();
  const { loader } = useProject();
  const { canCurrentUserCreatePage, createPage } = usePageStore(EPageStoreType.PROJECT);
  // handle page create
  const handleCreatePage = async () => {
    setIsCreatingPage(true);

    const payload: Partial<TPage> = {
      access: pageType === "private" ? EPageAccess.PRIVATE : EPageAccess.PUBLIC,
    };

    try {
      const res = await createPage(payload);
      if (!res?.id || !workspaceSlugParam || !projectIdParam) {
        setToast({
          type: TOAST_TYPE.ERROR,
          title: "Error!",
          message: "Page could not be created. Please try again.",
        });
        return;
      }

      const pageUrl = `/${workspaceSlugParam}/projects/${projectIdParam}/pages/${res.id}`;
      router.push(pageUrl);
    } catch (err: unknown) {
      const message =
        typeof err === "object" && err !== null && "data" in err && typeof (err as { data?: unknown }).data === "object"
          ? ((err as { data?: { error?: unknown } }).data?.error as string | undefined)
          : undefined;
      setToast({
        type: TOAST_TYPE.ERROR,
        title: "Error!",
        message: message || "Page could not be created. Please try again.",
      });
    } finally {
      setIsCreatingPage(false);
    }
  };

  return (
    <Header>
      <Header.LeftItem>
        <Breadcrumbs isLoading={loader === "init-loader"}>
          <CommonProjectBreadcrumbs workspaceSlug={workspaceSlugParam} projectId={projectIdParam} />
          <Breadcrumbs.Item
            component={
              <BreadcrumbLink
                label="Pages"
                href={`/${workspaceSlugParam}/projects/${projectIdParam}/pages/`}
                icon={<PageIcon className="h-4 w-4 text-tertiary" />}
                isLast
              />
            }
            isLast
          />
        </Breadcrumbs>
      </Header.LeftItem>
      {canCurrentUserCreatePage && (
        <Header.RightItem>
          <Button variant="primary" size="lg" onClick={handleCreatePage} loading={isCreatingPage}>
            {isCreatingPage ? "Adding" : "Add page"}
          </Button>
        </Header.RightItem>
      )}
    </Header>
  );
});
