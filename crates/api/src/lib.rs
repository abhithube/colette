use api_key::ApiKeyApi;
use auth::AuthApi;
use axum::{
    Router,
    http::{HeaderValue, Method, header},
    middleware, routing,
};
use bookmark::BookmarkApi;
use collection::CollectionApi;
use common::{ApiError, BooleanOp, DateOp, TextOp, add_user_extension};
pub use common::{
    ApiState, Config as ApiConfig, OidcConfig as ApiOidcConfig, StorageConfig as ApiStorageConfig,
};
use config::ConfigApi;
use feed::FeedApi;
use feed_entry::FeedEntryApi;
use stream::StreamApi;
use subscription::SubscriptionApi;
use subscription_entry::SubscriptionEntryApi;
use tag::TagApi;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::{
    Modify, OpenApi,
    openapi::{
        Server,
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    },
};
use utoipa_scalar::{Scalar, Servable};

pub mod api_key;
mod auth;
mod bookmark;
mod collection;
mod common;
pub mod config;
mod feed;
mod feed_entry;
mod stream;
mod subscription;
mod subscription_entry;
mod tag;

const API_PREFIX: &str = "/api";

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "Colette API",
        description = "Public REST API for the Colette app. Supports OAuth 2.0 and API key authentication.",
        license(name = "MIT")
    ),
    nest(
        (path = "/apiKeys", api = ApiKeyApi),
        (path = "/auth", api = AuthApi),
        (path = "/bookmarks", api = BookmarkApi),
        (path = "/collections", api = CollectionApi),
        (path = "/config", api = ConfigApi),
        (path = "/feedEntries", api = FeedEntryApi),
        (path = "/feeds", api = FeedApi),
        (path = "/streams", api = StreamApi),
        (path = "/subscriptions", api = SubscriptionApi),
        (path = "/subscriptionEntries", api = SubscriptionEntryApi),
        (path = "/tags", api = TagApi),
    ),
    components(schemas(ApiError, TextOp, BooleanOp, DateOp)),
    security(("bearerAuth" = [])),
    modifiers(&Security)
)]
pub struct ApiDoc;

struct Security;

impl Modify for Security {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

pub fn create_openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.servers = Some(vec![Server::new(API_PREFIX)]);

    openapi
}

pub fn create_router(api_state: ApiState, origin_urls: Option<Vec<String>>) -> Router {
    let openapi = create_openapi();

    let mut api = Router::new()
        .nest(
            API_PREFIX,
            Router::new()
                .nest("/apiKeys", ApiKeyApi::router())
                .nest("/auth", AuthApi::router())
                .nest("/bookmarks", BookmarkApi::router())
                .nest("/collections", CollectionApi::router())
                .nest("/feedEntries", FeedEntryApi::router())
                .nest("/feeds", FeedApi::router())
                .nest("/streams", StreamApi::router())
                .nest("/subscriptionEntries", SubscriptionEntryApi::router())
                .nest("/subscriptions", SubscriptionApi::router())
                .nest("/tags", TagApi::router())
                .layer(middleware::from_fn_with_state(
                    api_state.clone(),
                    add_user_extension,
                ))
                .nest("/config", ConfigApi::router()),
        )
        .merge(Scalar::with_url(
            format!("{}/doc", API_PREFIX),
            openapi.clone(),
        ))
        .route(
            &format!("{}/openapi.yaml", API_PREFIX),
            routing::get(|| async move { openapi.to_yaml().unwrap() }),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(api_state);

    if let Some(origin_urls) = origin_urls {
        let origins = origin_urls
            .iter()
            .filter_map(|e| e.parse::<HeaderValue>().ok())
            .collect::<Vec<_>>();

        api = api.layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                .allow_origin(origins)
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]),
        )
    }

    api
}
