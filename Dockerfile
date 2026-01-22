# Base stage for cargo-chef
FROM rust:1.93-alpine AS chef
RUN apk add --no-cache musl-dev gcc pkgconfig openssl-dev libx11-dev libxext-dev libxi-dev libxtst-dev libxrandr-dev libxcursor-dev libxcomposite-dev libxdamage-dev
RUN cargo install cargo-chef
WORKDIR /app

# Planner stage
FROM chef AS planner
COPY . .
RUN cargo chef prepare --bin server --recipe-path recipe.json

# Builder stage - Cache dependencies
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching layer
RUN cargo chef cook --release --bin server --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --bin server

# Runtime stage
FROM alpine:latest
WORKDIR /app
RUN apk add --no-cache libx11 libxi libxtst libxrandr libxcursor libxcomposite libxdamage
COPY --from=builder /app/target/release/server /usr/local/bin/sync_clipboard_server

# Environment variables with defaults
ENV SYNCCLIPBOARD_SERVER_PORT=5033
ENV SYNCCLIPBOARD_SERVER_HOST=0.0.0.0

EXPOSE 5033
CMD ["sync_clipboard_server"]
