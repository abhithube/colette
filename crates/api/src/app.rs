use axum::{
    http::{header, HeaderValue, Method},
    routing, Router,
};
use axum_embed::{FallbackBehavior, ServeEmbed};
use colette_config::Config;
use tower_http::cors::CorsLayer;
use tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::{
    auth::Api as Auth,
    bookmarks::Api as Bookmarks,
    common::{
        BaseError, BookmarkList, Context, EntryList, FeedDetectedList, FeedList, ProfileList,
    },
    entries::Api as Entries,
    feeds::Api as Feeds,
    profiles::Api as Profiles,
    tags::Api as Tags,
};

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../../packages/web/dist"]
struct Asset;

#[derive(utoipa::OpenApi)]
#[openapi(
    servers(
        (url = "http://localhost:8000/api/v1")
    ),
    nest(
        (path = "/auth", api = Auth),
        (path = "/bookmarks", api = Bookmarks),
        (path = "/entries", api = Entries),
        (path = "/feeds", api = Feeds),
        (path = "/profiles", api = Profiles),
        (path = "/tags", api = Tags)
    ),
    components(schemas(BaseError, BookmarkList, FeedDetectedList, EntryList, FeedList, ProfileList))
)]
struct ApiDoc;

pub struct App<'a> {
    state: Context,
    config: &'a Config,
    store: PostgresStore,
}

impl<'a> App<'a> {
    pub fn new(state: Context, config: &'a Config, store: PostgresStore) -> Self {
        Self {
            state,
            config,
            store,
        }
    }

    pub fn build_router(self) -> Router {
        let mut app = Router::new()
            .nest(
                "/api/v1",
                Router::new()
                    .merge(Scalar::with_url("/doc", ApiDoc::openapi()))
                    .route(
                        "/openapi.json",
                        routing::get(|| async { ApiDoc::openapi().to_pretty_json().unwrap() }),
                    )
                    .merge(Auth::router())
                    .merge(Bookmarks::router())
                    .merge(Entries::router())
                    .merge(Feeds::router())
                    .merge(Profiles::router())
                    .merge(Tags::router())
                    .with_state(self.state),
            )
            .fallback_service(ServeEmbed::<Asset>::with_parameters(
                Some(String::from("index.html")),
                FallbackBehavior::Ok,
                None,
            ))
            .layer(
                SessionManagerLayer::new(self.store)
                    .with_secure(false)
                    .with_expiry(Expiry::OnInactivity(Duration::days(1))),
            );

        if !self.config.origin_urls.is_empty() {
            let origins = self
                .config
                .origin_urls
                .iter()
                .filter_map(|e| e.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>();

            app = app.layer(
                CorsLayer::new()
                    .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                    .allow_origin(origins)
                    .allow_headers([header::CONTENT_TYPE])
                    .allow_credentials(true),
            )
        }

        app
    }
}
