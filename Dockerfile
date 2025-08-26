FROM rust:1.89-alpine AS base
WORKDIR /app
RUN apk add --no-cache musl-dev
RUN rustup default nightly
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
RUN cargo install --locked cargo-chef

FROM base AS prepare
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS builder
ARG TARGETPLATFORM
COPY --from=prepare /app/recipe.json recipe.json
RUN if [ "$TARGETPLATFORM" = "linux/amd64" ]; then echo "x86_64-unknown-linux-musl" > /tmp/target; \
    elif [ "$TARGETPLATFORM" = "linux/arm64" ]; then echo "aarch64-unknown-linux-musl" > /tmp/target; \
    else echo "Unsupported platform: $TARGETPLATFORM" >&2; exit 1; \
    fi
RUN TARGET=$(cat /tmp/target) && cargo chef cook --release --recipe-path recipe.json --target $TARGET
COPY . .
RUN TARGET=$(cat /tmp/target) && cargo build --release --target $TARGET
RUN TARGET=$(cat /tmp/target) && mkdir -p /app/$TARGETPLATFORM && cp target/$TARGET/release/colette-* /app/$TARGETPLATFORM/
RUN ls -lR /app/$TARGETPLATFORM

FROM gcr.io/distroless/static AS api
ARG TARGETPLATFORM
COPY --from=builder /app/$TARGETPLATFORM/colette-api /app/colette-api
EXPOSE 8000
ENTRYPOINT ["/app/colette-api"]

FROM gcr.io/distroless/static AS worker
ARG TARGETPLATFORM
COPY --from=builder /app/$TARGETPLATFORM/colette-worker /app/colette-worker
ENTRYPOINT ["/app/colette-worker"]