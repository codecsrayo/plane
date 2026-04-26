import { defineConfig } from "@playwright/test";
import { createBaseConfig } from "@plane/e2e-utils/setup/base-config";
import * as path from "path";
import * as url from "url";
import * as dotenv from "dotenv";

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));
dotenv.config({ path: path.join(__dirname, ".env") });

export default defineConfig({
  ...createBaseConfig(__dirname),
  testDir: "./e2e",
  projects: [
    {
      name: "web-session",
      testMatch: [
        "auth.spec.ts",
        "workspaces.spec.ts",
        "projects.spec.ts",
        "issues.spec.ts",
        "issues-extras.spec.ts",
        "cycles.spec.ts",
        "modules.spec.ts",
        "pages.spec.ts",
        "intake.spec.ts",
        "views.spec.ts",
        "notifications.spec.ts",
        "webhooks.spec.ts",
        "analytics.spec.ts",
        "search.spec.ts",
        "api-tokens.spec.ts",
        "states.spec.ts",
        "labels.spec.ts",
        "estimates.spec.ts",
        "users.spec.ts",
        "permissions.spec.ts",
        "contracts.spec.ts",
        "assets.spec.ts",
        "feature-flags.spec.ts",
      ],
    },
    {
      name: "web-v1",
      testMatch: ["api-v1.spec.ts"],
      use: { storageState: undefined },
    },
  ],
});
