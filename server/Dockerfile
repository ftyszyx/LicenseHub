# Stage 1: Build
FROM rust:1.92-slim-bookworm AS builder
ARG APP_NAME=app_server
ARG USE_ALIYUN_DEBIAN_MIRROR=1
ARG USE_ALIYUN_CRATES_MIRROR=1
ENV CARGO_HOME=/usr/local/cargo
# Install build dependencies, including postgresql client and nodejs/npm for swagger-ui
RUN if [ "$USE_ALIYUN_DEBIAN_MIRROR" = "1" ]; then sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources; fi
RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev nodejs npm curl

WORKDIR /usr/src/app
RUN if [ "$USE_ALIYUN_CRATES_MIRROR" = "1" ]; then \
    mkdir -p .cargo && \
    printf '[source.crates-io]\nreplace-with = "aliyun"\n\n[source.aliyun]\nregistry = "sparse+https://mirrors.aliyun.com/crates.io-index/"\n' > .cargo/config.toml; \
    fi
# Copy entire workspace and build
# Install sqlx-cli for running migrations at runtime
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release --workspace  \
    && mkdir -p /usr/src/out \
    && cp target/release/${APP_NAME} /usr/src/out/${APP_NAME}
RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres

FROM debian:bookworm-slim AS runner
ARG APP_NAME=app_server
ARG USE_ALIYUN_DEBIAN_MIRROR=1
WORKDIR /app
# Install required runtime libraries for the binary (OpenSSL, certs)
RUN if [ "$USE_ALIYUN_DEBIAN_MIRROR" = "1" ]; then sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources; fi \
 && apt-get update \
 # --no-install-recommends  只安装必要的依赖包
 && apt-get install -y --no-install-recommends pkg-config libssl-dev libpq-dev nodejs npm curl \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/out/${APP_NAME} .
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY migrations/ ./migrations/
COPY entrypoint.sh ./entrypoint.sh
RUN chmod +x ./entrypoint.sh
EXPOSE 3000
CMD ["./entrypoint.sh"]