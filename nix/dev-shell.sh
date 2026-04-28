#!/usr/bin/env bash
# Wrapper para entrar al entorno de desarrollo Nix del proyecto.
# Usa el flake en la raíz del repositorio (../flake.nix relativo a este script).
#
# Uso:
#   nix/dev-shell.sh              # Entra al shell interactivo
#   nix/dev-shell.sh cargo build  # Ejecuta un comando dentro del entorno Nix
#   nix/dev-shell.sh pnpm install

set -euo pipefail

# Resolver la raíz del repo a partir de la ubicación del script,
# para que funcione independientemente del cwd desde donde se invoque.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ $# -eq 0 ]; then
    echo "🚀 Entering Nix development environment..."
    exec nix develop "$REPO_ROOT"
else
    exec nix develop "$REPO_ROOT" --command "$@"
fi
