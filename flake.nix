{
  description = "Plane development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Rust toolchain - usando stable
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # Dependencias nativas requeridas para el proyecto
        nativeBuildInputs = with pkgs; [
          pkg-config
          rustToolchain
        ];
        extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
            "rustfmt"
          ];
        buildInputs = with pkgs; [
          # OpenSSL - soluciona el error de PKG_CONFIG_PATH
          openssl
          openssl.dev

          # PostgreSQL client libs (para sea-orm/sqlx)
          postgresql

          # Otras dependencias potenciales
          zlib

          # Node.js y pnpm (requerido: Node >=22.18.0, pnpm 10.32.1)
          nodejs_22
          corepack_22

          # Herramientas de desarrollo
          cargo-watch
          cargo-edit
          go-task   # Para ejecutar comandos del Taskfile.yml
          direnv    # Para carga automática del entorno Nix
        ];

        # Variables de entorno necesarias para cargo
        shellHook = ''
          export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
          export OPENSSL_DIR="${pkgs.openssl.dev}"
          export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
          export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"

          # Para postgresql
          export PG_CONFIG="${pkgs.postgresql}/bin/pg_config"

          # Nota: pnpm ya está habilitado vía corepack_22 en buildInputs
          # No es necesario ejecutar 'corepack enable pnpm' manualmente

          echo "✨ Plane development environment loaded"
          echo ""
          echo "📦 Rust toolchain:"
          echo "   Rust: $(rustc --version)"
          echo "   Cargo: $(cargo --version)"
          echo ""
          echo "🌐 Node.js ecosystem:"
          echo "   Node: $(node --version)"
          echo "   pnpm: $(pnpm --version 2>/dev/null || echo 'not enabled - run: corepack enable pnpm')"
          echo ""
          echo "🔧 Tools:"
          echo "   Task: $(task --version)"
          echo "   OpenSSL: ${pkgs.openssl.version}"
        '';

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs shellHook;
        };
      }
    );
}
