use axum::{
    http::{header, HeaderValue, Method},
    routing, Router,
};
use axum_embed::{FallbackBehavior, ServeEmbed};
use colette_config::Config;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::{
    auth::Api as Auth,
    bookmarks::Api as Bookmarks,
    collections::Api as Collections,
    common::{
        BaseError, BookmarkList, CollectionList, Context, EntryList, FeedDetectedList, FeedList,
        ProfileList, TagListUpdate,
    },
    entries::Api as Entries,
    feeds::Api as Feeds,
    profiles::Api as Profiles,
    tags::Api as Tags,
    SessionDatabase,
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
        (path = "/collections", api = Collections),
        (path = "/entries", api = Entries),
        (path = "/feeds", api = Feeds),
        (path = "/profiles", api = Profiles),
        (path = "/tags", api = Tags)
    ),
    components(schemas(BaseError, BookmarkList, CollectionList, FeedDetectedList, EntryList, FeedList, ProfileList, TagListUpdate))
)]
struct ApiDoc;

pub struct App {
    state: Context,
    config: Config,
    session_store: SessionDatabase,
}

impl App {
    pub fn new(state: Context, config: Config, session_store: SessionDatabase) -> Self {
        Self {
            state,
            config,
            session_store,
        }
    }

    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
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
                    .merge(Collections::router())
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
                SessionManagerLayer::new(self.session_store)
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

        let listener =
            TcpListener::bind(format!("{}:{}", self.config.host, self.config.port)).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
