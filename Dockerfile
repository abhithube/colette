ARG TARGET="x86_64-unknown-linux-musl"

FROM node:23-alpine AS web-build
WORKDIR /app
COPY package*.json tsconfig*.json ./
COPY apps/web ./apps/web
COPY packages ./packages
RUN npm ci
RUN npm run build --workspace=@colette/web

FROM rust:1.85-alpine AS base
WORKDIR /app
ARG TARGET
RUN apk add --no-cache musl-dev
RUN rustup target add $TARGET
RUN rustup default nightly
RUN cargo install cargo-chef

FROM base AS prepare
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS rust-build
COPY --from=prepare /app/recipe.json recipe.json
RUN cargo chef cook --target $TARGET --release --recipe-path recipe.json -p colette-server
COPY . .
COPY --from=web-build /app/apps/web/dist /app/web/dist
RUN cargo build --target $TARGET --release -p colette-server

FROM gcr.io/distroless/static AS release
ARG TARGET
COPY --from=rust-build /app/target/$TARGET/release/colette-server /
VOLUME /app/data
ENV DATA_DIR=/app/data
EXPOSE 8000
ENTRYPOINT ["/colette-server"]
