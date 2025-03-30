use std::ops::RangeFull;

use api_key::ApiKeyApi;
use auth::AuthApi;
use axum::{
    Router,
    http::{HeaderValue, Method, header},
    middleware, routing,
};
use bookmark::BookmarkApi;
use collection::CollectionApi;
pub use common::ApiState;
use common::{
    BaseError, BooleanOp, DateOp, TextOp, add_connection_info_extension, add_user_extension,
};
use feed::FeedApi;
use feed_entry::FeedEntryApi;
use stream::StreamApi;
use subscription::SubscriptionApi;
use subscription_entry::SubscriptionEntryApi;
use tag::TagApi;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::{OpenApi, openapi::Server};
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

mod api_key;
mod auth;
mod bookmark;
mod collection;
mod common;
mod feed;
mod feed_entry;
mod stream;
mod subscription;
mod subscription_entry;
mod tag;

#[derive(utoipa::OpenApi)]
#[openapi(components(schemas(BaseError, TextOp, BooleanOp, DateOp)))]
struct ApiDoc;

pub fn create_router(api_state: ApiState, origin_urls: Option<Vec<String>>) -> Router {
    let api_prefix = "/api";

    let (api, mut openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest(
            api_prefix,
            OpenApiRouter::new()
                .nest("/apiKeys", ApiKeyApi::router())
                .nest("/auth", AuthApi::router())
                .nest("/bookmarks", BookmarkApi::router())
                .nest("/collections", CollectionApi::router())
                .nest("/feedEntries", FeedEntryApi::router())
                .nest("/feeds", FeedApi::router())
                .nest("/streams", StreamApi::router())
                .nest("/subscriptionEntries", SubscriptionEntryApi::router())
                .nest("/subscriptions", SubscriptionApi::router())
                .nest("/tags", TagApi::router()),
        )
        .split_for_parts();

    openapi.info.title = "Colette API".to_owned();
    openapi.servers = Some(vec![Server::new(api_prefix)]);

    openapi.paths.paths = openapi
        .paths
        .paths
        .drain(RangeFull)
        .map(|(k, v)| (k.replace(&format!("{}/", api_prefix), "/"), v))
        .collect();

    let mut api = api
        .merge(Scalar::with_url(
            format!("{}/doc", api_prefix),
            openapi.clone(),
        ))
        .route(
            &format!("{}/openapi.json", api_prefix),
            routing::get(|| async move { openapi.to_pretty_json().unwrap() }),
        )
        .layer(middleware::from_fn_with_state(
            api_state.clone(),
            add_user_extension,
        ))
        .layer(middleware::from_fn(add_connection_info_extension))
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
                .allow_headers([header::CONTENT_TYPE])
                .allow_credentials(true),
        )
    }

    api
}
