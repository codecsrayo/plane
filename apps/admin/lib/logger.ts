/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

/**
 * Centralized logger for the admin application.
 *
 * In production builds all log levels below `error` are suppressed.
 * `error` is always forwarded so error-tracking integrations (Sentry, etc.)
 * can hook into `console.error` as usual.
 *
 * Usage:
 *   import { logger } from "@/lib/logger";
 *   logger.error("Something went wrong", error);
 *   logger.warn("Heads up", detail);
 *   logger.info("Done", result);   // dev-only
 *   logger.debug("Trace", value);  // dev-only
 */

const IS_PRODUCTION = import.meta.env.PROD;

type LogArgs = [message: string, ...rest: unknown[]];

function createLogger() {
  return {
    /** Always logged – use for caught exceptions and unexpected failures. */
    error(...args: LogArgs): void {
      console.error(...args);
    },

    /** Logged in all environments. Use for recoverable edge-cases. */
    warn(...args: LogArgs): void {
      if (!IS_PRODUCTION) {
        console.warn(...args);
      }
    },

    /** Development-only informational messages. */
    info(...args: LogArgs): void {
      if (!IS_PRODUCTION) {
        console.info(...args);
      }
    },

    /** Development-only verbose trace messages. */
    debug(...args: LogArgs): void {
      if (!IS_PRODUCTION) {
        console.debug(...args);
      }
    },
  };
}

export const logger = createLogger();
