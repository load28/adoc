# syntax=docker/dockerfile:1.7

FROM rust:1.90.0-bookworm AS rust-builder
WORKDIR /workspace
COPY Cargo.toml Cargo.lock ./
COPY apps ./apps
COPY crates ./crates
COPY tools ./tools
COPY docs/design/contracts ./docs/design/contracts
COPY infra/migrations ./infra/migrations
RUN cargo build --locked --release -p adoc-api -p adoc-worker

FROM oven/bun:1.3.13-debian AS web-builder
WORKDIR /workspace
COPY package.json bun.lock tsconfig.base.json ./
COPY apps/web ./apps/web
COPY packages ./packages
RUN bun install --frozen-lockfile
RUN bun run --cwd apps/web build

FROM debian:bookworm-slim AS rust-runtime
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 adoc \
    && useradd --uid 10001 --gid 10001 --no-create-home --shell /usr/sbin/nologin adoc
USER 10001:10001

FROM rust-runtime AS api
ARG ADOC_RELEASE_SHA=development
ARG ADOC_RELEASE_VERSION=1.0.0
LABEL org.opencontainers.image.revision=$ADOC_RELEASE_SHA \
      org.opencontainers.image.version=$ADOC_RELEASE_VERSION \
      org.opencontainers.image.title="adoc-api"
COPY --from=rust-builder /workspace/target/release/adoc-api /usr/local/bin/adoc-api
ENTRYPOINT ["/usr/local/bin/adoc-api"]

FROM rust-runtime AS worker
ARG ADOC_RELEASE_SHA=development
ARG ADOC_RELEASE_VERSION=1.0.0
LABEL org.opencontainers.image.revision=$ADOC_RELEASE_SHA \
      org.opencontainers.image.version=$ADOC_RELEASE_VERSION \
      org.opencontainers.image.title="adoc-worker"
COPY --from=rust-builder /workspace/target/release/adoc-worker /usr/local/bin/adoc-worker
ENTRYPOINT ["/usr/local/bin/adoc-worker"]

FROM oven/bun:1.3.13-debian AS web
ARG ADOC_RELEASE_SHA=development
ARG ADOC_RELEASE_VERSION=1.0.0
LABEL org.opencontainers.image.revision=$ADOC_RELEASE_SHA \
      org.opencontainers.image.version=$ADOC_RELEASE_VERSION \
      org.opencontainers.image.title="adoc-web"
WORKDIR /app
COPY --from=web-builder --chown=bun:bun /workspace/apps/web/dist ./dist
USER bun
CMD ["bun", "run", "dist/web-runtime/runtime.js"]
