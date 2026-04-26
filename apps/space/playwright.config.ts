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
      name: "space-session",
      testMatch: ["auth-spaces.spec.ts", "spaces-deploy-boards.spec.ts"],
    },
  ],
});
