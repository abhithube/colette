use std::sync::Arc;

use axum::{body::Body, http::Response};
use axum_embed::{FallbackBehavior, ServeEmbed};
use colette_api::{
    auth::AuthState, backup::BackupState, bookmark::BookmarkState, feed::FeedState,
    feed_entry::FeedEntryState, smart_feed::SmartFeedState, tag::TagState, Api, ApiState,
};
use colette_backup::{netscape::NetscapeManager, opml::OpmlManager};
use colette_core::{
    auth::AuthService, backup::BackupService, bookmark::BookmarkService, feed::FeedService,
    feed_entry::FeedEntryService, smart_feed::SmartFeedService, tag::TagService,
};
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_queue::cloudflare::CloudflareQueue;
use colette_repository::d1::{
    D1BackupRepository, D1BookmarkRepository, D1FeedEntryRepository, D1FeedRepository,
    D1SmartFeedRepository, D1TagRepository, D1UserRepository,
};
use colette_scraper::{
    bookmark::DefaultBookmarkScraper, downloader::DefaultDownloader, feed::DefaultFeedScraper,
};
use colette_session::kv::KvSessionStore;
use colette_task::{import_bookmarks, import_feeds};
use colette_util::{base64::Base64Encoder, password::ArgonHasher};
use time::Duration;
use tower::Service;
use tower_sessions::{Expiry, SessionManagerLayer};
use worker::{Context, Env, HttpRequest};

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../web/dist"]
struct Asset;

#[worker::event(fetch)]
async fn fetch(req: HttpRequest, env: Env, _ctx: Context) -> worker::Result<Response<Body>> {
    console_error_panic_hook::set_once();

    let d1 = Arc::new(env.d1("DB")?);
    let kv = env.kv("KV")?;
    let queue = env.queue("QUEUE")?;
    let api_prefix = env.var("API_PREFIX")?.to_string();

    let backup_repository = D1BackupRepository::new(d1.clone());
    let bookmark_repository = D1BookmarkRepository::new(d1.clone());
    let feed_repository = D1FeedRepository::new(d1.clone());

    let client = colette_http::Client::build(None).unwrap();
    let downloader = DefaultDownloader::new(client.clone());

    let base64_encoder = Base64Encoder;

    let auth_service = Arc::new(AuthService::new(
        D1UserRepository::new(d1.clone()),
        ArgonHasher,
    ));
    let backup_service = Arc::new(BackupService::new(
        D1BackupRepository::new(d1.clone()),
        feed_repository.clone(),
        bookmark_repository.clone(),
        OpmlManager,
        NetscapeManager,
    ));
    let bookmark_service = Arc::new(BookmarkService::new(
        bookmark_repository,
        register_bookmark_plugins(
            client.clone(),
            downloader.clone(),
            DefaultBookmarkScraper::new(downloader),
        ),
        base64_encoder.clone(),
    ));
    let feed_service = Arc::new(FeedService::new(
        feed_repository,
        register_feed_plugins(
            client,
            downloader.clone(),
            DefaultFeedScraper::new(downloader.clone()),
        ),
    ));
    let feed_entry_service = Arc::new(FeedEntryService::new(
        D1FeedEntryRepository::new(d1.clone()),
        base64_encoder,
    ));
    let smart_feed_service = Arc::new(SmartFeedService::new(D1SmartFeedRepository::new(
        d1.clone(),
    )));
    let tag_service = Arc::new(TagService::new(D1TagRepository::new(d1.clone())));

    let api_state = ApiState::new(
        AuthState::new(auth_service),
        BackupState::new(
            backup_service,
            Arc::new(CloudflareQueue::<import_feeds::Data>::new(queue.clone())),
            Arc::new(CloudflareQueue::<import_bookmarks::Data>::new(queue)),
        ),
        BookmarkState::new(bookmark_service),
        FeedState::new(feed_service),
        FeedEntryState::new(feed_entry_service),
        SmartFeedState::new(smart_feed_service),
        TagState::new(tag_service),
    );

    let mut router = Api::new(&api_state, &api_prefix)
        .build()
        .with_state(api_state)
        .layer(
            SessionManagerLayer::new(KvSessionStore::new(kv))
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(Duration::days(1))),
        )
        .fallback_service(ServeEmbed::<Asset>::with_parameters(
            Some(String::from("index.html")),
            FallbackBehavior::Ok,
            None,
        ));

    let resp = router.call(req).await?;

    Ok(resp)
}
