ARG TARGET="aarch64-unknown-linux-musl"

FROM node AS frontend-build
WORKDIR /app
COPY package*.json .
COPY packages ./packages
RUN npm ci
RUN npm run build --workspace=@colette/web

FROM clux/muslrust AS base 
WORKDIR /app
RUN cargo install cargo-chef 

FROM base AS prepare
COPY rs .
RUN cargo chef prepare  --recipe-path recipe.json

FROM base AS rust-build
ARG TARGET
COPY --from=prepare /app/recipe.json recipe.json
RUN cargo chef cook --target $TARGET --release --recipe-path recipe.json
COPY rs .
COPY --from=frontend-build /app/packages/web/dist ./packages/web/dist
RUN cargo build --target $TARGET --release

FROM gcr.io/distroless/static AS release
VOLUME ["/data"]
ENV DATABASE_URL="sqlite:///data/sqlite.db?mode=rwc"
ARG TARGET
EXPOSE 8000
COPY --from=rust-build /app/target/$TARGET/release/colette-server /
ENTRYPOINT ["/colette-server"]
