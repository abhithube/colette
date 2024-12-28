ARG TARGET="x86_64-unknown-linux-musl"

FROM rust:1.83-alpine AS base
WORKDIR /app
ARG TARGET
RUN apk add --no-cache musl-dev npm
RUN npm i -D tailwindcss
RUN wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-$TARGET.tgz
RUN tar -xvf cargo-binstall-$TARGET.tgz
RUN cp cargo-binstall /usr/local/cargo/bin
RUN cargo binstall cargo-chef
RUN cargo binstall cargo-leptos
RUN rustup target add $TARGET

FROM base AS prepare
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS build
COPY --from=prepare /app/recipe.json recipe.json
RUN cargo chef cook --target $TARGET --release --recipe-path recipe.json -p colette-leptos
COPY . .
RUN rustup target add wasm32-unknown-unknown
RUN cargo leptos build --release -p colette-leptos

FROM gcr.io/distroless/static AS release
ARG TARGET
COPY --from=build /app/target/release/colette-leptos /
COPY --from=build /app/target/site ./site
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT="/site"
EXPOSE 3000
ENTRYPOINT ["/colette-leptos"]
