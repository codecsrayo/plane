/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { enableStaticRendering } from "mobx-react";
// plane types
import type { IUser, IWorkspace, IInstanceInfo } from "@plane/types";
// stores
import type { IInstanceStore } from "./instance.store";
import { InstanceStore } from "./instance.store";
import type { IThemeStore } from "./theme.store";
import { ThemeStore } from "./theme.store";
import type { IUserStore } from "./user.store";
import { UserStore } from "./user.store";
import type { IWorkspaceStore } from "./workspace.store";
import { WorkspaceStore } from "./workspace.store";

enableStaticRendering(typeof window === "undefined");

/**
 * Shape of the server-side hydration payload passed to `RootStore.hydrate`.
 * All fields are optional — partial hydration is valid on SSR/SSG pages.
 */
export type TRootStoreHydrateData = {
  theme?: "dark" | "light";
  instance?: IInstanceInfo;
  user?: IUser;
  workspace?: Record<string, IWorkspace>;
};

export class RootStore {
  theme: IThemeStore;
  instance: IInstanceStore;
  user: IUserStore;
  workspace: IWorkspaceStore;

  constructor() {
    this.theme = new ThemeStore(this);
    this.instance = new InstanceStore(this);
    this.user = new UserStore(this);
    this.workspace = new WorkspaceStore(this);
  }

  hydrate(initialData: TRootStoreHydrateData) {
    this.theme.hydrate(initialData.theme);
    this.instance.hydrate(initialData.instance as IInstanceInfo);
    this.user.hydrate(initialData.user);
    this.workspace.hydrate(initialData.workspace ?? {});
  }

  resetOnSignOut() {
    localStorage.setItem("theme", "system");
    this.instance = new InstanceStore(this);
    this.user = new UserStore(this);
    this.theme = new ThemeStore(this);
    this.workspace = new WorkspaceStore(this);
  }
}
