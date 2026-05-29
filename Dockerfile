# ── Stage 1: Build ─────────────────────────────────────────────────────────────
#
# We use the full Rust toolchain image.  All system-level libraries needed at
# compile time (ring / rustls crypto) are available without extra apt steps.
FROM rust:1.96-slim-bookworm AS builder

WORKDIR /app

# ── Cache dependency compilation ──────────────────────────────────────────────
# Copy manifests and lock file first so this layer is reused on source changes.
COPY Cargo.toml Cargo.lock ./

# Create minimal stub sources that satisfy the workspace members list.
# This lets cargo compile all deps without touching our real code.
COPY foundation/types/Cargo.toml  foundation/types/Cargo.toml
COPY foundation/events/Cargo.toml foundation/events/Cargo.toml
COPY domain/quests/Cargo.toml     domain/quests/Cargo.toml
COPY domain/world/Cargo.toml      domain/world/Cargo.toml
COPY domain/agents/Cargo.toml     domain/agents/Cargo.toml
COPY runtime/sandbox/Cargo.toml   runtime/sandbox/Cargo.toml
COPY runtime/server/Cargo.toml    runtime/server/Cargo.toml

RUN mkdir -p \
    foundation/types/src foundation/events/src \
    domain/quests/src domain/world/src domain/agents/src \
    runtime/sandbox/src runtime/server/src && \
    for d in foundation/types foundation/events domain/quests domain/world domain/agents runtime/sandbox; do \
        echo "// stub" > $d/src/lib.rs; \
    done && \
    echo "fn main(){}" > runtime/server/src/main.rs && \
    cargo build --release --bin server 2>&1 | tail -3 && \
    rm -rf foundation/*/src domain/*/src runtime/*/src

# ── Build the real binary ─────────────────────────────────────────────────────
COPY foundation/ ./foundation/
COPY domain/     ./domain/
COPY runtime/    ./runtime/

RUN touch runtime/server/src/main.rs && \
    cargo build --release --bin server && \
    strip target/release/server

# ── Stage 2: Runtime ───────────────────────────────────────────────────────────
#
# Minimal image: only the binary and CA certs for outbound HTTPS (OpenAI API).
# Migrations are embedded in the binary by sqlx::migrate!() at compile time.
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates wget \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --system --no-create-home --shell /usr/sbin/nologin appuser

WORKDIR /app
COPY --from=builder /app/target/release/server ./server

USER appuser

EXPOSE 8080

HEALTHCHECK \
    --interval=30s \
    --timeout=5s \
    --start-period=15s \
    --retries=3 \
    CMD ["wget", "-qO-", "http://localhost:8080/health"]

ENTRYPOINT ["/app/server"]
