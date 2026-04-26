/**
 * Feature flags y claves de configuración de instancia.
 * Extraídas de `src/routes/instances.rs` → función `config_var()`.
 *
 * Campos:
 *   - `key`        : nombre del flag en `/api/instances/configurations`
 *   - `group`      : categoría (AUTHENTICATION, GITHUB, etc.)
 *   - `encrypted`  : si el backend redacta el valor en respuestas (true = secreto)
 *   - `requiresFlag`: si el endpoint relacionado falla cuando el flag = "0" / vacío
 */
export interface ConfigVar {
  key: string;
  group: string;
  encrypted: boolean;
}

export const CONFIG_VARS: ConfigVar[] = [
  // AUTHENTICATION — afectan /auth/* sign-in, sign-up, magic-link
  { key: "ENABLE_SIGNUP",              group: "AUTHENTICATION", encrypted: false },
  { key: "ENABLE_EMAIL_PASSWORD",      group: "AUTHENTICATION", encrypted: false },
  { key: "ENABLE_MAGIC_LINK_LOGIN",    group: "AUTHENTICATION", encrypted: false },
  { key: "DISABLE_WORKSPACE_CREATION", group: "WORKSPACE_MANAGEMENT", encrypted: false },

  // GOOGLE — requiere IS_GOOGLE_ENABLED="1" para /auth/google
  { key: "IS_GOOGLE_ENABLED",   group: "GOOGLE", encrypted: false },
  { key: "GOOGLE_CLIENT_ID",    group: "GOOGLE", encrypted: false },
  { key: "GOOGLE_CLIENT_SECRET",group: "GOOGLE", encrypted: true  },
  { key: "ENABLE_GOOGLE_SYNC",  group: "GOOGLE", encrypted: false },

  // GITHUB — requiere IS_GITHUB_ENABLED="1" para /auth/github
  // IS_GITHUB_INTEGRATION_ENABLED="1" para workspace-integrations github
  { key: "IS_GITHUB_ENABLED",             group: "GITHUB", encrypted: false },
  { key: "GITHUB_CLIENT_ID",              group: "GITHUB", encrypted: false },
  { key: "GITHUB_CLIENT_SECRET",          group: "GITHUB", encrypted: true  },
  { key: "GITHUB_ORGANIZATION_ID",        group: "GITHUB", encrypted: false },
  { key: "ENABLE_GITHUB_SYNC",            group: "GITHUB", encrypted: false },
  { key: "IS_GITHUB_INTEGRATION_ENABLED", group: "GITHUB", encrypted: false },
  { key: "GITHUB_APP_NAME",               group: "GITHUB", encrypted: false },
  { key: "GITHUB_APP_ID",                 group: "GITHUB", encrypted: false },
  { key: "GITHUB_APP_PRIVATE_KEY",        group: "GITHUB", encrypted: true  },
  { key: "GITHUB_WEBHOOK_SECRET",         group: "GITHUB", encrypted: true  },

  // SLACK
  { key: "IS_SLACK_ENABLED",    group: "SLACK", encrypted: false },
  { key: "SLACK_CLIENT_ID",     group: "SLACK", encrypted: false },
  { key: "SLACK_CLIENT_SECRET", group: "SLACK", encrypted: true  },

  // GITLAB — requiere IS_GITLAB_ENABLED="1" para /auth/gitlab
  { key: "IS_GITLAB_ENABLED",             group: "GITLAB", encrypted: false },
  { key: "IS_GITLAB_INTEGRATION_ENABLED", group: "GITLAB", encrypted: false },
  { key: "GITLAB_HOST",                   group: "GITLAB", encrypted: false },
  { key: "GITLAB_CLIENT_ID",              group: "GITLAB", encrypted: false },
  { key: "GITLAB_CLIENT_SECRET",          group: "GITLAB", encrypted: true  },
  { key: "ENABLE_GITLAB_SYNC",            group: "GITLAB", encrypted: false },

  // GITEA — requiere IS_GITEA_ENABLED="1" para /auth/gitea
  { key: "IS_GITEA_ENABLED",    group: "GITEA", encrypted: false },
  { key: "GITEA_HOST",          group: "GITEA", encrypted: false },
  { key: "GITEA_CLIENT_ID",     group: "GITEA", encrypted: false },
  { key: "GITEA_CLIENT_SECRET", group: "GITEA", encrypted: true  },
  { key: "ENABLE_GITEA_SYNC",   group: "GITEA", encrypted: false },

  // SMTP — requiere ENABLE_SMTP="1" para envío de emails
  { key: "ENABLE_SMTP",          group: "SMTP", encrypted: false },
  { key: "EMAIL_HOST",           group: "SMTP", encrypted: false },
  { key: "EMAIL_HOST_USER",      group: "SMTP", encrypted: false },
  { key: "EMAIL_HOST_PASSWORD",  group: "SMTP", encrypted: true  },
  { key: "EMAIL_PORT",           group: "SMTP", encrypted: false },
  { key: "EMAIL_FROM",           group: "SMTP", encrypted: false },
  { key: "EMAIL_USE_TLS",        group: "SMTP", encrypted: false },
  { key: "EMAIL_USE_SSL",        group: "SMTP", encrypted: false },

  // AI — requiere LLM_API_KEY para /api/workspaces/{slug}/ai-assistant
  { key: "LLM_API_KEY",   group: "AI", encrypted: true  },
  { key: "LLM_PROVIDER",  group: "AI", encrypted: false },
  { key: "LLM_MODEL",     group: "AI", encrypted: false },
  { key: "GPT_ENGINE",    group: "AI", encrypted: false },

  // UNSPLASH — requiere UNSPLASH_ACCESS_KEY para /api/unsplash
  { key: "UNSPLASH_ACCESS_KEY", group: "UNSPLASH", encrypted: true },

  // INTERCOM
  { key: "IS_INTERCOM_ENABLED", group: "INTERCOM", encrypted: false },
  { key: "INTERCOM_APP_ID",     group: "INTERCOM", encrypted: false },
];

/** Flags que bloquean un endpoint cuando están desactivados ("0" o vacío). */
export const FLAG_GATED_ENDPOINTS: Record<string, string[]> = {
  ENABLE_SIGNUP:              ["POST /auth/sign-up", "POST /auth/spaces/sign-up"],
  ENABLE_EMAIL_PASSWORD:      ["POST /auth/sign-in"],
  ENABLE_MAGIC_LINK_LOGIN:    ["POST /auth/magic-generate", "POST /auth/magic-sign-in"],
  DISABLE_WORKSPACE_CREATION: ["POST /api/workspaces"],
  IS_GOOGLE_ENABLED:          ["GET /auth/google", "GET /auth/google/callback"],
  IS_GITHUB_ENABLED:          ["GET /auth/github/callback"],
  IS_GITHUB_INTEGRATION_ENABLED: ["GET/POST /api/workspaces/{slug}/workspace-integrations"],
  IS_GITLAB_ENABLED:          ["GET /auth/gitlab", "GET /auth/gitlab/callback"],
  IS_GITEA_ENABLED:           ["GET /auth/gitea", "GET /auth/gitea/callback"],
  LLM_API_KEY:                ["POST /api/workspaces/{slug}/ai-assistant", "POST .../rephrase-grammar"],
  UNSPLASH_ACCESS_KEY:        ["GET /api/unsplash"],
};

/** Claves cuyo valor el backend redacta (no las devuelve en texto plano). */
export const ENCRYPTED_KEYS = CONFIG_VARS
  .filter((v) => v.encrypted)
  .map((v) => v.key);

/** Keys de autenticación sin cifrar — útil para test de smoke de /configurations. */
export const AUTH_FLAG_KEYS = CONFIG_VARS
  .filter((v) => v.group === "AUTHENTICATION")
  .map((v) => v.key);
