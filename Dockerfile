ARG RUST_VERSION=1.83.0
ARG APP_NAME=messenger
FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG APP_NAME
WORKDIR /app

RUN apt-get update && apt-get install -y curl --no-install-recommends && \
    rm -rf /var/lib/apt/lists/*

# Build the application.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=bind,source=migrations,target=migrations \
    <<EOF
set -e
cargo build --locked --release
cp ./target/release/$APP_NAME /bin/server
EOF

################################################################################
FROM debian:bullseye-slim AS final

ARG UID=10001

COPY --from=build /bin/server /bin/

EXPOSE 8000

CMD ["/bin/server"]
