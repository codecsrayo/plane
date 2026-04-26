import type { PlaywrightTestConfig } from "@playwright/test";
import * as path from "path";
import * as url from "url";

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));

const API_BASE = process.env.API_BASE_URL ?? "http://localhost:8000";

/**
 * Fábrica de configuración base compartida.
 * Cada app llama a esta función y extiende con sus propios projects/testMatch.
 *
 * @param appDir  __dirname del playwright.config.ts de la app
 */
export function createBaseConfig(appDir: string): PlaywrightTestConfig {
  const storagePath = path.join(appDir, "state.json");

  return {
    fullyParallel: false,
    workers: process.env.CI ? 1 : 2,
    retries: process.env.CI ? 1 : 0,
    timeout: 30_000,
    reporter: [
      ["list"],
      ["html", { outputFolder: path.join(appDir, "playwright-report"), open: "never" }],
    ],

    // setup/teardown compartidos del package
    globalSetup: path.resolve(__dirname, "global-setup.ts"),
    globalTeardown: path.resolve(__dirname, "global-teardown.ts"),

    use: {
      baseURL: API_BASE,
      storageState: storagePath,
    },
  };
}
