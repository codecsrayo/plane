/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useEffect } from "react";
import type { ReactNode } from "react";
import { observer } from "mobx-react";
import { useSearchParams, usePathname } from "next/navigation";
import useSWR from "swr";
// components
import { LogoSpinner } from "@/components/common/logo-spinner";
// helpers
import { EPageTypes, sanitizeNextPath } from "@/helpers/authentication.helper";
// hooks
import { useWorkspace } from "@/hooks/store/use-workspace";
import { useUser, useUserProfile, useUserSettings } from "@/hooks/store/user";
import { useAppRouter } from "@/hooks/use-app-router";

type TPageType = EPageTypes;

type TAuthenticationWrapper = {
  children: ReactNode;
  pageType?: TPageType;
};

type TRedirectConfig =
  | {
      href: string;
      method: "push" | "replace";
    }
  | undefined;

export const AuthenticationWrapper = observer(function AuthenticationWrapper(props: TAuthenticationWrapper) {
  const pathname = usePathname();
  const router = useAppRouter();
  const searchParams = useSearchParams();
  const nextPath = searchParams.get("next_path");
  // props
  const { children, pageType = EPageTypes.AUTHENTICATED } = props;
  // hooks
  const { isLoading: isUserLoading, data: currentUser, fetchCurrentUser } = useUser();
  const { data: currentUserProfile } = useUserProfile();
  const { data: currentUserSettings } = useUserSettings();
  const { loader: workspacesLoader, workspaces } = useWorkspace();

  const { isLoading: isUserSWRLoading } = useSWR("USER_INFORMATION", async () => await fetchCurrentUser(), {
    revalidateOnFocus: false,
    shouldRetryOnError: false,
  });

  const isUserOnboard =
    currentUserProfile?.is_onboarded ||
    (currentUserProfile?.onboarding_step?.profile_complete &&
      currentUserProfile?.onboarding_step?.workspace_create &&
      currentUserProfile?.onboarding_step?.workspace_invite &&
      currentUserProfile?.onboarding_step?.workspace_join) ||
    false;

  const getValidNextPath = (): string | undefined => {
    return sanitizeNextPath(
      nextPath,
      Object.values(workspaces || {}).map((workspace) => workspace.slug)
    );
  };

  const getWorkspaceRedirectionUrl = (): string => {
    let redirectionRoute = "/create-workspace";
    const validNextPath = getValidNextPath();

    // validating the nextPath from the router query
    if (validNextPath) {
      redirectionRoute = validNextPath;
      return redirectionRoute;
    }

    // validate the last and fallback workspace_slug
    const currentWorkspaceSlug =
      currentUserSettings?.workspace?.last_workspace_slug || currentUserSettings?.workspace?.fallback_workspace_slug;

    // validate the current workspace_slug is available in the user's workspace list
    const isCurrentWorkspaceValid = Object.values(workspaces || {}).findIndex(
      (workspace) => workspace.slug === currentWorkspaceSlug
    );

    if (isCurrentWorkspaceValid >= 0) redirectionRoute = `/${currentWorkspaceSlug}`;

    return redirectionRoute;
  };

  const getLoginRedirectUrl = (): string => {
    const sanitizedSearchParams = new URLSearchParams(searchParams.toString());
    sanitizedSearchParams.delete("next_path");
    const sanitizedSearch = sanitizedSearchParams.toString();
    const currentPath = `${pathname}${sanitizedSearch ? `?${sanitizedSearch}` : ""}`;

    return `/${currentPath ? `?next_path=${encodeURIComponent(currentPath)}` : ""}`;
  };

  const getRedirectConfig = (): TRedirectConfig => {
    if (pageType === EPageTypes.PUBLIC) return undefined;

    if (pageType === EPageTypes.NON_AUTHENTICATED) {
      if (!currentUser?.id) return undefined;

      if (currentUserProfile?.id && isUserOnboard) {
        return {
          href: getWorkspaceRedirectionUrl(),
          method: "push",
        };
      }

      return {
        href: "/onboarding",
        method: "push",
      };
    }

    if (pageType === EPageTypes.ONBOARDING) {
      if (!currentUser?.id) {
        return {
          href: getLoginRedirectUrl(),
          method: "push",
        };
      }

      if (currentUser && currentUserProfile?.id && isUserOnboard) {
        return {
          href: getWorkspaceRedirectionUrl(),
          method: "replace",
        };
      }

      return undefined;
    }

    if (pageType === EPageTypes.SET_PASSWORD) {
      if (!currentUser?.id) {
        return {
          href: getLoginRedirectUrl(),
          method: "push",
        };
      }

      if (currentUser && !currentUser?.is_password_autoset && currentUserProfile?.id && isUserOnboard) {
        return {
          href: getWorkspaceRedirectionUrl(),
          method: "push",
        };
      }

      return undefined;
    }

    if (pageType === EPageTypes.AUTHENTICATED) {
      if (!currentUser?.id) {
        return {
          href: getLoginRedirectUrl(),
          method: "push",
        };
      }

      if (!currentUserProfile?.id || !isUserOnboard) {
        return {
          href: "/onboarding",
          method: "push",
        };
      }
    }

    return undefined;
  };

  const redirectConfig = getRedirectConfig();
  const redirectHref = redirectConfig?.href;
  const redirectMethod = redirectConfig?.method;

  useEffect(() => {
    if (!redirectHref || !redirectMethod) return;

    if (redirectMethod === "replace") {
      router.replace(redirectHref);
      return;
    }

    router.push(redirectHref);
  }, [redirectHref, redirectMethod, router]);

  if ((isUserSWRLoading || isUserLoading || workspacesLoader) && !currentUser?.id)
    return (
      <div className="relative flex h-screen w-full items-center justify-center">
        <LogoSpinner />
      </div>
    );

  if (pageType === EPageTypes.PUBLIC) return <>{children}</>;

  if (redirectConfig) return <></>;

  return <>{children}</>;
});
