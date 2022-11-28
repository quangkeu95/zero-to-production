# Leveraging the pre-built Docker images with 
# cargo-chef and the Rust toolchain
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS cacher 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
FROM rust as builder
WORKDIR /app
COPY . .
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
ENV SQLX_OFFLINE true
RUN cargo build --release 

# Release
FROM gcr.io/distroless/cc-debian11 as runtime
# FROM debian:bullseye-slim AS runtime
# WORKDIR /app
# RUN apt-get update -y \
#   && apt-get install -y --no-install-recommends openssl ca-certificates \
#   # Clean up
#   && apt-get autoremove -y \
#   && apt-get clean -y \
#   && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/zero2prod /app/zero2prod
COPY configuration /app/configuration
ENV APP_ENVIRONMENT production

EXPOSE 8000
ENTRYPOINT ["/app/zero2prod"]