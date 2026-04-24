#!/usr/bin/env bash
# Wrapper script to enter the Nix development environment
# Usage: ./dev-shell.sh [command]
#
# Examples:
#   ./dev-shell.sh              # Enter interactive shell
#   ./dev-shell.sh cargo build  # Run cargo build in Nix env
#   ./dev-shell.sh pnpm install # Run pnpm install in Nix env

set -e

if [ $# -eq 0 ]; then
    echo "🚀 Entering Nix development environment..."
    exec nix develop
else
    exec nix develop --command "$@"
fi
