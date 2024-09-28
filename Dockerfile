ARG TARGET="x86_64-unknown-linux-musl"

FROM node:22-alpine AS web-build
WORKDIR /app
COPY package*.json .
COPY apps ./apps
COPY packages ./packages
RUN npm ci
RUN npm i -D typescript
RUN npm run build --workspace=@colette/web

FROM rust:1.80-alpine AS base
WORKDIR /app
ARG TARGET
RUN apk add --no-cache musl-dev
RUN rustup target add $TARGET
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
EXPOSE 8000
COPY --from=rust-build /app/target/$TARGET/release/colette-server /
ENTRYPOINT ["/colette-server"]
