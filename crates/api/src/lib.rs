use auth::AuthApi;
use axum::{
    Router,
    http::{HeaderValue, Method, header},
    middleware, routing,
};
use bookmark::BookmarkApi;
use collection::CollectionApi;
use common::{ApiError, verify_auth_extension};
pub use common::{
    ApiState, Config as ApiConfig, OidcConfig as ApiOidcConfig, S3Config as ApiS3Config,
    ServerConfig as ApiServerConfig,
};
use config::ConfigApi;
use entry::EntryApi;
use feed::FeedApi;
use subscription::SubscriptionApi;
use tag::TagApi;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use url::Url;
use utoipa::{
    Modify, OpenApi,
    openapi::{
        Server,
        security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
    },
};
use utoipa_scalar::{Scalar, Servable};

use crate::{backup::BackupApi, common::CreatedResource, pat::PersonalAccessTokensApi};

mod auth;
mod backup;
mod bookmark;
mod collection;
mod common;
pub mod config;
mod entry;
mod feed;
mod pagination;
mod pat;
mod subscription;
mod tag;

const API_PREFIX: &str = "/api";

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "Colette API",
        description = "Public REST API for the Colette app. Supports email OTP, OAuth 2.0, and PAT authentication.",
        license(name = "MIT")
    ),
    nest(
        (path = "/auth", api = AuthApi),
        (path = "/backups", api = BackupApi),
        (path = "/bookmarks", api = BookmarkApi),
        (path = "/collections", api = CollectionApi),
        (path = "/config", api = ConfigApi),
        (path = "/entries", api = EntryApi),
        (path = "/feeds", api = FeedApi),
        (path = "/pats", api = PersonalAccessTokensApi),
        (path = "/subscriptions", api = SubscriptionApi),
        (path = "/tags", api = TagApi),
    ),
    components(schemas(CreatedResource, ApiError)),
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
            );

            components.add_security_scheme(
                "apiKey",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-Api-Key"))),
            );
        }
    }
}

pub fn create_openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.servers = Some(vec![Server::new(API_PREFIX)]);

    openapi
}

pub fn create_router(api_state: ApiState, origin_urls: Option<Vec<Url>>) -> Router {
    let openapi = create_openapi();

    let public_router = Router::new()
        .nest("/auth", AuthApi::public())
        .nest("/config", ConfigApi::router())
        .nest("/feeds", FeedApi::router());

    let authenticated_router = Router::new()
        .nest("/auth", AuthApi::authenticated())
        .nest("/backups", BackupApi::router())
        .nest("/bookmarks", BookmarkApi::router())
        .nest("/collections", CollectionApi::router())
        .nest("/entries", EntryApi::router())
        .nest("/pats", PersonalAccessTokensApi::router())
        .nest("/subscriptions", SubscriptionApi::router())
        .nest("/tags", TagApi::router())
        .layer(middleware::from_fn_with_state(
            api_state.clone(),
            verify_auth_extension,
        ));

    let mut router = Router::new()
        .nest(
            API_PREFIX,
            Router::new()
                .merge(public_router)
                .merge(authenticated_router),
        )
        .merge(Scalar::with_url(
            format!("{API_PREFIX}/doc"),
            openapi.clone(),
        ))
        .route(
            &format!("{API_PREFIX}/openapi.yaml"),
            routing::get(|| async move { openapi.to_yaml().unwrap() }),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(api_state);

    if let Some(origin_urls) = origin_urls {
        let origins = origin_urls
            .iter()
            .filter_map(|e| e.origin().ascii_serialization().parse::<HeaderValue>().ok())
            .collect::<Vec<_>>();

        router = router.layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                .allow_origin(origins)
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
                .allow_credentials(true),
        )
    }

    router
}
