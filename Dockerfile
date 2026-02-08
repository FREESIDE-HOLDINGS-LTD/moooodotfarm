# syntax=docker/dockerfile:1.6
FROM rustlang/rust:nightly-slim AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y protobuf-compiler
COPY . .
WORKDIR /build/moooodotfarm-backend
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/moooodotfarm-backend/target \
    cargo build --release --bin moooodotfarm \
    && mkdir -p /build/artifacts \
    && cp /build/moooodotfarm-backend/target/release/moooodotfarm /build/artifacts/

FROM debian:trixie-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -m -u 1000 moooo
COPY --from=builder /build/artifacts/moooodotfarm /usr/local/bin/moooodotfarm
USER moooo
ENTRYPOINT ["/usr/local/bin/moooodotfarm"]
CMD ["run", "/config.toml"]
