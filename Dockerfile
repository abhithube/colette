FROM node:24-alpine AS web
WORKDIR /app
COPY package.json tsconfig*.json ./
COPY apps/web ./apps/web
COPY packages ./packages
RUN npm i
RUN cd apps/web && npx vite build

FROM rust:1.87-alpine AS base
WORKDIR /app
RUN apk add --no-cache musl-dev
RUN rustup default nightly
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
RUN cargo install --locked cargo-chef

FROM base AS prepare
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS build
ARG TARGETPLATFORM
COPY --from=prepare /app/recipe.json recipe.json
COPY --from=web /app/apps/web/dist /app/web/dist
COPY . .
RUN set -e && \
    if [ "$TARGETPLATFORM" = "linux/amd64" ]; then TARGET="x86_64-unknown-linux-musl"; \
    elif [ "$TARGETPLATFORM" = "linux/arm64" ]; then TARGET="aarch64-unknown-linux-musl"; \
    else echo "Unsupported platform: $TARGETPLATFORM" >&2; exit 1; fi && \
    \
    cargo chef cook --release --recipe-path recipe.json --target $TARGET -p colette-server && \
    \
    cargo build --release --target $TARGET -p colette-server && \
    \
    mkdir -p /app/linux && cp target/$TARGET/release/colette-server /app/$TARGETPLATFORM

FROM gcr.io/distroless/static AS release
ARG TARGETPLATFORM
COPY --from=build /app/$TARGETPLATFORM /app/colette-server
VOLUME /app/data
ENV DATA_DIR=/app/data
EXPOSE 8000
ENTRYPOINT ["/app/colette-server"]
