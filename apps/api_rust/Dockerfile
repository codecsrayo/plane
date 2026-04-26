# ─────────────────────────────────────────────────────────────────────────────
# Dockerfile — API Rust (reemplaza Django)
# Build context: raíz del repositorio (plane/)
#
# Stage 1 (builder): compila el binario en release con caché de dependencias.
# Stage 2 (runtime): imagen mínima Debian con sólo el binario y las libs.
# ─────────────────────────────────────────────────────────────────────────────

# ── Stage 1: Builder ──────────────────────────────────────────────────────────
FROM rust:1.94-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        libpq-dev \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Usa el OpenSSL del sistema (libssl-dev) enlazado estáticamente.
# Sustituye al feature "vendored" (que compilaba OpenSSL desde fuente con Perl)
# manteniendo la portabilidad del binario sin la dependencia de Perl.
ENV OPENSSL_STATIC=1

WORKDIR /build

# ── Caché de dependencias ─────────────────────────────────────────────────────
# Copiar sólo manifiestos para cachear el layer de compilación de deps.
# Cuando sólo cambia src/ este layer no se invalida.
COPY apps/api_rust/.cargo ./.cargo
COPY apps/api_rust/Cargo.toml apps/api_rust/Cargo.lock ./
COPY apps/api_rust/migration/Cargo.toml ./migration/

RUN mkdir -p src migration/src && \
    echo 'fn main() {}' > src/main.rs && \
    echo '// stub' > migration/src/lib.rs && \
    echo 'fn main() {}' > migration/src/main.rs

# Compilar deps en release (la capa más costosa — se cachea).
# Sin `|| true`: si los manifiestos no resuelven aquí, no debe seguir el build.
RUN cargo build --release --bin api_rust

# Limpiar fuentes-stub. Importante: borrar también los artefactos del crate
# `migration` para que la copia real (con lib.rs poblado) se recompile desde
# cero. Sin esto, cargo ha cacheado metadata de un lib vacío y el binario
# final no encuentra `migration::Migrator` (E0432) — paquete vacío reusado.
RUN rm -rf src migration/src \
    target/release/deps/migration-* \
    target/release/deps/libmigration-* \
    target/release/.fingerprint/migration-*

# ── Código fuente ─────────────────────────────────────────────────────────────
COPY apps/api_rust/src ./src
COPY apps/api_rust/migration/src ./migration/src

# `touch` solo es defensivo; el `rm -rf` anterior ya garantiza recompilación
# de migration. El touch sobre src/main.rs hace lo propio para el bin final.
RUN touch src/main.rs migration/src/main.rs migration/src/lib.rs

RUN cargo build --release --bin api_rust

# ── Stage 2: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
        libssl3 \
        libpq5 \
        ca-certificates \
        curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1001 -s /bin/sh plane

WORKDIR /app

COPY --from=builder /build/target/release/api_rust ./api_rust

RUN chmod +x ./api_rust && chown -R plane:plane /app

USER plane

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD curl -sf http://localhost:8000/api/health || exit 1

CMD ["./api_rust"]
