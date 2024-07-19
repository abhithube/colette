ARG TARGET="aarch64-unknown-linux-musl"

FROM node:22-alpine AS web-build
WORKDIR /app
COPY package*.json .
COPY packages ./packages
RUN npm ci
RUN npm i -D typescript
RUN npm run build --workspace=@colette/web

FROM rust:1.79-alpine AS base
WORKDIR /app/rs
ARG TARGET
RUN apk add --no-cache musl-dev make zlib-dev
RUN wget https://download.gnome.org/sources/libxml2/2.13/libxml2-2.13.2.tar.xz && \
  tar xf libxml2-2.13.2.tar.xz && \
  cd libxml2-2.13.2 && \
  ./configure --with-minimum --with-html --with-xpath --enable-static && \
  make && make install
RUN rustup target add $TARGET
RUN cargo install cargo-chef

FROM base AS prepare
COPY rs .
RUN cargo chef prepare  --recipe-path recipe.json

FROM base AS rust-build
COPY --from=prepare /app/rs/recipe.json recipe.json
RUN cargo chef cook --target $TARGET --release --recipe-path recipe.json
COPY rs .
COPY --from=web-build /app/packages/web/dist ../packages/web/dist
RUN cargo build --target $TARGET --release

FROM gcr.io/distroless/static AS release
ARG TARGET
EXPOSE 8000
COPY --from=rust-build /app/rs/target/$TARGET/release/colette-server /
ENTRYPOINT ["/colette-server"]
