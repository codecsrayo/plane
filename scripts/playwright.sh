#!/usr/bin/env bash
# scripts/playwright.sh — Wrapper Playwright con carga consistente de .env.e2e
#
# Uso:
#   scripts/playwright.sh <app> [args...]
#
# Ejemplos:
#   scripts/playwright.sh web --headed auth.spec.ts
#   scripts/playwright.sh admin --project=admin-session
#   scripts/playwright.sh web codegen http://localhost:3000
#
# El primer arg es la app (web|admin|space). Si es "codegen", se invoca
# `playwright codegen` en lugar de `playwright test`.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP="${1:?APP requerido (web|admin|space)}"
shift

case "$APP" in
  web|admin|space) ;;
  *) echo "APP inválida: $APP (usar web|admin|space)" >&2; exit 2 ;;
esac

# Carga .env.e2e si existe (no falla si no existe)
if [ -f "$ROOT_DIR/.env.e2e" ]; then
  set -a
  # shellcheck disable=SC1090,SC1091
  . "$ROOT_DIR/.env.e2e"
  set +a
fi

cd "$ROOT_DIR/apps/$APP"

# Permite primer arg = "codegen" para mapear a `playwright codegen`
if [ "${1:-}" = "codegen" ]; then
  shift
  exec pnpm exec playwright codegen "$@"
fi

exec pnpm exec playwright test "$@"
