use std::sync::Arc;

use axum::{body::Body, http::Response};
use colette_api::{
    auth::AuthState, backup::BackupState, bookmark::BookmarkState, feed::FeedState,
    feed_entry::FeedEntryState, profile::ProfileState, smart_feed::SmartFeedState, tag::TagState,
    Api, ApiState,
};
use colette_backup::{netscape::NetscapeManager, opml::OpmlManager};
use colette_core::{
    auth::AuthService, backup::BackupService, bookmark::BookmarkService, feed::FeedService,
    feed_entry::FeedEntryService, profile::ProfileService, smart_feed::SmartFeedService,
    tag::TagService,
};
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_queue::cloudflare::CloudflareQueue;
use colette_repository::d1::{
    D1BackupRepository, D1BookmarkRepository, D1FeedEntryRepository, D1FeedRepository,
    D1ProfileRepository, D1SmartFeedRepository, D1TagRepository, D1UserRepository,
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

#[worker::event(fetch)]
async fn fetch(req: HttpRequest, env: Env, _ctx: Context) -> worker::Result<Response<Body>> {
    console_error_panic_hook::set_once();

    let d1 = Arc::new(env.d1("DB")?);
    let kv = env.kv("KV")?;
    let queue = env.queue("QUEUE")?;
    let api_prefix = env.var("API_PREFIX")?.to_string();

    let backup_repository = Box::new(D1BackupRepository::new(d1.clone()));
    let bookmark_repository = Box::new(D1BookmarkRepository::new(d1.clone()));
    let feed_repository = Box::new(D1FeedRepository::new(d1.clone()));
    let feed_entry_repository = Box::new(D1FeedEntryRepository::new(d1.clone()));
    let profile_repository = Box::new(D1ProfileRepository::new(d1.clone()));
    let smart_feed_repository = Box::new(D1SmartFeedRepository::new(d1.clone()));
    let tag_repository = Box::new(D1TagRepository::new(d1.clone()));
    let user_repository = Box::new(D1UserRepository::new(d1));

    let client = colette_http::Client::build(None).unwrap();
    let downloader = Box::new(DefaultDownloader::new(client.clone()));
    let feed_scraper = Box::new(DefaultFeedScraper::new(downloader.clone()));
    let bookmark_scraper = Box::new(DefaultBookmarkScraper::new(downloader.clone()));
    let feed_plugin_registry = Box::new(register_feed_plugins(downloader.clone(), feed_scraper));
    let bookmark_plugin_registry = Box::new(register_bookmark_plugins(client, bookmark_scraper));

    let base64_encoder = Box::new(Base64Encoder);

    let feed_service = FeedService::new(feed_repository.clone(), feed_plugin_registry.clone());

    let import_feeds_queue = Box::new(CloudflareQueue::<import_feeds::Data>::new(queue.clone()));
    let import_bookmarks_queue = Box::new(CloudflareQueue::<import_bookmarks::Data>::new(queue));

    let api_state = ApiState::new(
        AuthState::new(AuthService::new(
            user_repository,
            profile_repository.clone(),
            Box::new(ArgonHasher),
        )),
        BackupState::new(
            BackupService::new(
                backup_repository,
                feed_repository.clone(),
                bookmark_repository.clone(),
                Box::new(OpmlManager),
                Box::new(NetscapeManager),
            ),
            import_feeds_queue,
            import_bookmarks_queue,
        ),
        BookmarkState::new(BookmarkService::new(
            bookmark_repository,
            bookmark_plugin_registry,
            base64_encoder.clone(),
        )),
        FeedState::new(feed_service),
        FeedEntryState::new(FeedEntryService::new(feed_entry_repository, base64_encoder)),
        ProfileState::new(ProfileService::new(profile_repository)),
        SmartFeedState::new(SmartFeedService::new(smart_feed_repository)),
        TagState::new(TagService::new(tag_repository)),
    );

    let mut router = Api::new(&api_state, &api_prefix)
        .build()
        .with_state(api_state)
        .layer(
            SessionManagerLayer::new(KvSessionStore::new(kv))
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(Duration::days(1))),
        );

    let resp = router.call(req).await?;

    Ok(resp)
}
