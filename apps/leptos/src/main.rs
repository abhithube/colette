#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::{future::Future, pin::Pin};

    use colette_api::{
        auth::AuthState, backup::BackupState, bookmark::BookmarkState, feed::FeedState,
        feed_entry::FeedEntryState, profile::ProfileState, smart_feed::SmartFeedState,
        tag::TagState, Api, ApiState,
    };
    use colette_backup::{netscape::NetscapeManager, opml::OpmlManager};
    use colette_core::{
        auth::AuthService, backup::BackupService, bookmark::BookmarkService, feed::FeedService,
        feed_entry::FeedEntryService, profile::ProfileService, scraper::ScraperService,
        smart_feed::SmartFeedService, tag::TagService,
    };
    use colette_leptos::{app::*, AppState};
    use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
    use colette_queue::memory::InMemoryQueue;
    use colette_repository::postgres::{
        PostgresBackupRepository, PostgresBookmarkRepository, PostgresFeedEntryRepository,
        PostgresFeedRepository, PostgresProfileRepository, PostgresScraperRepository,
        PostgresSmartFeedRepository, PostgresTagRepository, PostgresUserRepository,
    };
    use colette_scraper::{
        bookmark::DefaultBookmarkScraper, downloader::DefaultDownloader, feed::DefaultFeedScraper,
    };
    use colette_session::postgres::PostgresSessionStore;
    use colette_task::{
        import_bookmarks, import_feeds, refresh_feeds, scrape_bookmark, scrape_feed,
    };
    use colette_util::{base64::Base64Encoder, password::ArgonHasher};
    use colette_worker::{run_cron_worker, run_task_worker};
    use deadpool_postgres::{tokio_postgres::NoTls, Config, Runtime};
    use http::{header, HeaderValue, Method};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tokio::net::TcpListener;
    use tower::ServiceBuilder;
    use tower_http::{cors::CorsLayer, trace::TraceLayer};
    use tower_sessions::{cookie::time::Duration, ExpiredDeletion, Expiry, SessionManagerLayer};
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    #[cfg(debug_assertions)]
    {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
    #[cfg(not(debug_assertions))]
    {
        use tracing_subscriber::Layer;
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
            )
            .init();
    }

    let app_config = colette_config::load_config().unwrap();

    let mut config = Config::new();
    config.url = Some(app_config.database_url);
    let pool = config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    let backup_repository = Box::new(PostgresBackupRepository::new(pool.clone()));
    let bookmark_repository = Box::new(PostgresBookmarkRepository::new(pool.clone()));
    let feed_repository = Box::new(PostgresFeedRepository::new(pool.clone()));
    let feed_entry_repository = Box::new(PostgresFeedEntryRepository::new(pool.clone()));
    let profile_repository = Box::new(PostgresProfileRepository::new(pool.clone()));
    let scraper_repository = Box::new(PostgresScraperRepository::new(pool.clone()));
    let smart_feed_repository = Box::new(PostgresSmartFeedRepository::new(pool.clone()));
    let tag_repository = Box::new(PostgresTagRepository::new(pool.clone()));
    let user_repository = Box::new(PostgresUserRepository::new(pool.clone()));
    let session_store = PostgresSessionStore::new(pool);

    let client = colette_http::Client::build(None, None).unwrap();
    let downloader = Box::new(DefaultDownloader::new(client.clone()));
    let feed_scraper = Box::new(DefaultFeedScraper::new(downloader.clone()));
    let bookmark_scraper = Box::new(DefaultBookmarkScraper::new(downloader.clone()));
    let feed_plugin_registry = Box::new(register_feed_plugins(downloader.clone(), feed_scraper));
    let bookmark_plugin_registry = Box::new(register_bookmark_plugins(client, bookmark_scraper));

    let base64_encoder = Box::new(Base64Encoder);

    let feed_service = FeedService::new(feed_repository.clone(), feed_plugin_registry.clone());
    let scraper_service = ScraperService::new(
        scraper_repository,
        feed_plugin_registry,
        bookmark_plugin_registry.clone(),
    );

    let (scrape_feed_queue, scrape_feed_receiver) = InMemoryQueue::new();
    let var_name = InMemoryQueue::new();
    let (scrape_bookmark_queue, scrape_bookmark_receiver) = var_name;
    let (import_feeds_queue, import_feeds_receiver) = InMemoryQueue::new();
    let (import_bookmarks_queue, import_bookmarks_receiver) = InMemoryQueue::new();

    let scrape_feed_queue = Box::new(scrape_feed_queue);
    let scrape_bookmark_queue = Box::new(scrape_bookmark_queue);
    let import_feeds_queue = Box::new(import_feeds_queue);
    let import_bookmarks_queue = Box::new(import_bookmarks_queue);

    let scrape_feed_task = ServiceBuilder::new()
        .concurrency_limit(5)
        .service(scrape_feed::Task::new(scraper_service.clone()));
    let scrape_bookmark_task = ServiceBuilder::new()
        .concurrency_limit(5)
        .service(scrape_bookmark::Task::new(scraper_service));
    let refresh_feeds_task =
        refresh_feeds::Task::new(feed_service.clone(), scrape_feed_queue.clone());
    let import_feeds_task = import_feeds::Task::new(scrape_feed_queue);
    let import_bookmarks_task = import_bookmarks::Task::new(scrape_bookmark_queue);

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

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    let app_state = AppState {
        api_state: api_state.clone(),
        leptos_options,
    };

    let routes = generate_route_list(App);

    let mut app = Api::new(&api_state, &app_config.api_prefix)
        .build()
        .with_state(api_state)
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let api_state = app_state.api_state.clone();
                move || provide_context(api_state.clone())
            },
            {
                let leptos_options = app_state.leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .layer(
            SessionManagerLayer::new(session_store.clone())
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(Duration::days(1))),
        )
        .layer(TraceLayer::new_for_http())
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(app_state);

    if !app_config.origin_urls.is_empty() {
        let origins = app_config
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

    log!("listening on http://{}", &addr);

    let listener = TcpListener::bind(&addr).await.unwrap();

    let server = async { axum::serve(listener, app).await };

    let refresh_task_worker = if app_config.refresh_enabled {
        Box::pin(run_cron_worker(
            &app_config.cron_refresh,
            refresh_feeds_task,
        ))
    } else {
        Box::pin(std::future::ready(())) as Pin<Box<dyn Future<Output = ()> + Send>>
    };

    let _ = tokio::join!(
        server,
        run_task_worker(scrape_feed_receiver, scrape_feed_task),
        run_task_worker(scrape_bookmark_receiver, scrape_bookmark_task),
        run_task_worker(import_feeds_receiver, import_feeds_task),
        run_task_worker(import_bookmarks_receiver, import_bookmarks_task),
        refresh_task_worker,
        session_store.continuously_delete_expired(tokio::time::Duration::from_secs(60))
    );
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
