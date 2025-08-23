use std::{error::Error, net::SocketAddr, sync::Arc};

use axum_embed::{FallbackBehavior, ServeEmbed};
use chrono::Duration;
use colette_api::{ApiConfig, ApiOidcConfig, ApiS3Config, ApiServerConfig, ApiState};
use colette_core::{
    auth::{
        BuildAuthorizationUrlHandler, CreatePatHandler, DeletePatHandler, ExchangeCodeHandler,
        GetPatHandler, GetUserHandler, JwtConfig, ListPatsHandler, OidcConfig,
        RefreshAccessTokenHandler, SendOtpHandler, UpdatePatHandler, ValidateAccessTokenHandler,
        ValidatePatHandler, VerifyOtpHandler,
    },
    backup::{ExportBackupHandler, ImportBackupHandler},
    bookmark::{
        ArchiveThumbnailHandler, CreateBookmarkHandler, DeleteBookmarkHandler,
        ExportBookmarksHandler, GetBookmarkHandler, ImportBookmarksHandler,
        LinkBookmarkTagsHandler, ListBookmarksHandler, RefreshBookmarkHandler,
        ScrapeBookmarkHandler, UpdateBookmarkHandler,
    },
    collection::{
        CreateCollectionHandler, DeleteCollectionHandler, GetCollectionHandler,
        ListCollectionsHandler, UpdateCollectionHandler,
    },
    feed::{DetectFeedsHandler, GetFeedHandler, ListFeedsHandler, RefreshFeedHandler},
    feed_entry::{GetFeedEntryHandler, ListFeedEntriesHandler},
    subscription::{
        CreateSubscriptionHandler, DeleteSubscriptionHandler, ExportSubscriptionsHandler,
        GetSubscriptionHandler, ImportSubscriptionsHandler, LinkSubscriptionTagsHandler,
        ListSubscriptionsHandler, UpdateSubscriptionHandler,
    },
    subscription_entry::{
        GetSubscriptionEntryHandler, ListSubscriptionEntriesHandler,
        MarkSubscriptionEntryAsReadHandler, MarkSubscriptionEntryAsUnreadHandler,
    },
    tag::{CreateTagHandler, DeleteTagHandler, GetTagHandler, ListTagsHandler, UpdateTagHandler},
};
use colette_http::ReqwestClient;
use colette_job::{
    archive_thumbnail::ArchiveThumbnailJobHandler, import_bookmarks::ImportBookmarksJobHandler,
    refresh_feeds::RefreshFeedsJobHandler, scrape_bookmark::ScrapeBookmarkJobHandler,
    scrape_feed::ScrapeFeedJobHandler,
};
use colette_jwt::JwtManagerImpl;
use colette_oidc::OidcClientImpl;
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_queue::TokioQueue;
use colette_repository::{
    PostgresBackupRepository, PostgresBookmarkRepository, PostgresCollectionRepository,
    PostgresFeedEntryRepository, PostgresFeedRepository, PostgresPatRepository,
    PostgresSubscriptionEntryRepository, PostgresSubscriptionRepository, PostgresTagRepository,
    PostgresUserRepository,
};
use colette_s3::S3ClientImpl;
use colette_scraper::{bookmark::BookmarkScraper, feed::FeedScraper};
use colette_smtp::{SmtpClientImpl, SmtpConfig};
use sqlx::PgPool;
use tokio::{net::TcpListener, sync::Mutex};
use tower::{ServiceBuilder, ServiceExt};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use worker::{CronWorker, JobWorker};

mod config;
mod worker;

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../web/dist/"]
struct Asset;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

    let app_config = config::from_env().await?;

    let pool = PgPool::connect_lazy(&app_config.database.url)?;

    let bookmark_repository = PostgresBookmarkRepository::new(pool.clone());
    let collection_repository = PostgresCollectionRepository::new(pool.clone());
    let feed_entry_repository = PostgresFeedEntryRepository::new(pool.clone());
    let pat_repository = PostgresPatRepository::new(pool.clone());
    let subscription_repository = PostgresSubscriptionRepository::new(pool.clone());
    let subscription_entry_repository = PostgresSubscriptionEntryRepository::new(pool.clone());
    let tag_repository = PostgresTagRepository::new(pool.clone());

    let reqwest_client = reqwest::Client::builder().build()?;
    let http_client = ReqwestClient::new(reqwest_client.clone());

    let jwt_config = JwtConfig {
        secret: app_config.jwt.secret.into_bytes(),
        access_duration: Duration::minutes(15),
        refresh_duration: Duration::days(7),
    };
    let jwt_manager = JwtManagerImpl::new(&jwt_config.secret);

    let mut oidc_config = Option::<OidcConfig>::None;
    let mut oidc_client = Option::<OidcClientImpl>::None;
    if let Some(config) = app_config.oidc.clone() {
        oidc_config = Some(OidcConfig {
            issuer_url: config.issuer_url.clone(),
            client_id: config.client_id.clone(),
            redirect_uri: config.redirect_uri.clone(),
            scopes: config.scopes,
        });

        let client = OidcClientImpl::init(
            colette_oidc::OidcConfig {
                issuer_url: config.issuer_url,
                client_id: config.client_id,
                redirect_uri: config.redirect_uri,
            },
            reqwest_client.clone(),
        )
        .await?;

        oidc_client = Some(client);
    }

    let stmp_client = SmtpClientImpl::create(SmtpConfig {
        host: app_config.smtp.host,
        username: app_config.smtp.username,
        password: app_config.smtp.password,
        from_address: app_config.smtp.from_address,
    })?;

    let s3_client = S3ClientImpl::init(colette_s3::S3Config {
        access_key_id: app_config.s3.access_key_id,
        secret_access_key: app_config.s3.secret_access_key,
        region: app_config.s3.region,
        endpoint: app_config.s3.endpoint,
        bucket_name: app_config.s3.bucket_name,
        path_style_enabled: app_config.s3.path_style_enabled,
    })
    .await?;

    let (scrape_feed_producer, scrape_feed_consumer) = TokioQueue::new().split();
    let (scrape_bookmark_producer, scrape_bookmark_consumer) = TokioQueue::new().split();
    let (archive_thumbnail_producer, archive_thumbnail_consumer) = TokioQueue::new().split();
    let (import_bookmarks_producer, import_bookmarks_consumer) = TokioQueue::new().split();

    let bookmark_scraper = Arc::new(BookmarkScraper::new(
        http_client.clone(),
        register_bookmark_plugins(reqwest_client.clone()),
    ));

    let feed_repository = PostgresFeedRepository::new(pool.clone());
    let user_repository = PostgresUserRepository::new(pool.clone());

    let feed_scraper = Arc::new(FeedScraper::new(
        http_client.clone(),
        register_feed_plugins(reqwest_client),
    ));

    let list_bookmarks_handler = Arc::new(ListBookmarksHandler::new(
        bookmark_repository.clone(),
        collection_repository.clone(),
    ));
    let refresh_bookmark_handler = Arc::new(RefreshBookmarkHandler::new(
        bookmark_repository.clone(),
        bookmark_scraper.clone(),
    ));
    let archive_thumbnail_handler = Arc::new(ArchiveThumbnailHandler::new(
        bookmark_repository.clone(),
        http_client.clone(),
        s3_client,
    ));
    let list_feeds_handler = Arc::new(ListFeedsHandler::new(feed_repository.clone()));
    let refresh_feed_handler = Arc::new(RefreshFeedHandler::new(
        feed_repository.clone(),
        feed_entry_repository.clone(),
        feed_scraper.clone(),
    ));

    let mut api_state = ApiState {
        // Auth
        send_otp: Arc::new(SendOtpHandler::new(user_repository.clone(), stmp_client)),
        verify_otp: Arc::new(VerifyOtpHandler::new(
            user_repository.clone(),
            jwt_manager.clone(),
            jwt_config.clone(),
        )),
        build_authorization_url: None,
        exchange_code: None,
        get_user: Arc::new(GetUserHandler::new(user_repository.clone())),
        refresh_access_token: Arc::new(RefreshAccessTokenHandler::new(
            user_repository.clone(),
            jwt_manager.clone(),
            jwt_config.clone(),
        )),
        validate_access_token: Arc::new(ValidateAccessTokenHandler::new(jwt_manager.clone())),
        list_pats: Arc::new(ListPatsHandler::new(pat_repository.clone())),
        get_pat: Arc::new(GetPatHandler::new(pat_repository.clone())),
        create_pat: Arc::new(CreatePatHandler::new(user_repository.clone())),
        update_pat: Arc::new(UpdatePatHandler::new(user_repository.clone())),
        delete_pat: Arc::new(DeletePatHandler::new(user_repository.clone())),
        validate_pat: Arc::new(ValidatePatHandler::new(pat_repository)),

        // Backup
        import_backup: Arc::new(ImportBackupHandler::new(PostgresBackupRepository::new(
            pool,
        ))),
        export_backup: Arc::new(ExportBackupHandler::new(
            bookmark_repository.clone(),
            subscription_repository.clone(),
            tag_repository.clone(),
        )),

        // Bookmarks
        list_bookmarks: list_bookmarks_handler.clone(),
        get_bookmark: Arc::new(GetBookmarkHandler::new(bookmark_repository.clone())),
        create_bookmark: Arc::new(CreateBookmarkHandler::new(
            bookmark_repository.clone(),
            archive_thumbnail_producer.clone(),
        )),
        update_bookmark: Arc::new(UpdateBookmarkHandler::new(
            bookmark_repository.clone(),
            archive_thumbnail_producer.clone(),
        )),
        delete_bookmark: Arc::new(DeleteBookmarkHandler::new(
            bookmark_repository.clone(),
            archive_thumbnail_producer,
        )),
        scrape_bookmark: Arc::new(ScrapeBookmarkHandler::new(bookmark_scraper)),
        refresh_bookmark: refresh_bookmark_handler.clone(),
        link_bookmark_tags: Arc::new(LinkBookmarkTagsHandler::new(bookmark_repository.clone())),
        import_bookmarks: Arc::new(ImportBookmarksHandler::new(
            bookmark_repository.clone(),
            import_bookmarks_producer,
        )),
        export_bookmarks: Arc::new(ExportBookmarksHandler::new(bookmark_repository)),
        archive_thumbnail: archive_thumbnail_handler.clone(),

        // Collections
        list_collections: Arc::new(ListCollectionsHandler::new(collection_repository.clone())),
        get_collection: Arc::new(GetCollectionHandler::new(collection_repository.clone())),
        create_collection: Arc::new(CreateCollectionHandler::new(collection_repository.clone())),
        update_collection: Arc::new(UpdateCollectionHandler::new(collection_repository.clone())),
        delete_collection: Arc::new(DeleteCollectionHandler::new(collection_repository.clone())),

        // Feeds
        list_feeds: list_feeds_handler.clone(),
        get_feed: Arc::new(GetFeedHandler::new(feed_repository)),
        detect_feeds: Arc::new(DetectFeedsHandler::new(http_client, feed_scraper)),
        refresh_feed: refresh_feed_handler.clone(),

        // Feed Entries
        list_feed_entries: Arc::new(ListFeedEntriesHandler::new(feed_entry_repository.clone())),
        get_feed_entry: Arc::new(GetFeedEntryHandler::new(feed_entry_repository)),

        // Subscriptions
        list_subscriptions: Arc::new(ListSubscriptionsHandler::new(
            subscription_repository.clone(),
        )),
        get_subscription: Arc::new(GetSubscriptionHandler::new(subscription_repository.clone())),
        create_subscription: Arc::new(CreateSubscriptionHandler::new(
            subscription_repository.clone(),
        )),
        update_subscription: Arc::new(UpdateSubscriptionHandler::new(
            subscription_repository.clone(),
        )),
        delete_subscription: Arc::new(DeleteSubscriptionHandler::new(
            subscription_repository.clone(),
        )),
        link_subscription_tags: Arc::new(LinkSubscriptionTagsHandler::new(
            subscription_repository.clone(),
        )),
        import_subscriptions: Arc::new(ImportSubscriptionsHandler::new(
            subscription_repository.clone(),
        )),
        export_subscriptions: Arc::new(ExportSubscriptionsHandler::new(subscription_repository)),

        // Subscription Entries
        list_subscription_entries: Arc::new(ListSubscriptionEntriesHandler::new(
            subscription_entry_repository.clone(),
            collection_repository,
        )),
        get_subscription_entry: Arc::new(GetSubscriptionEntryHandler::new(
            subscription_entry_repository.clone(),
        )),
        mark_subscription_entry_as_read: Arc::new(MarkSubscriptionEntryAsReadHandler::new(
            subscription_entry_repository.clone(),
        )),
        mark_subscription_entry_as_unread: Arc::new(MarkSubscriptionEntryAsUnreadHandler::new(
            subscription_entry_repository,
        )),

        // Tags
        list_tags: Arc::new(ListTagsHandler::new(tag_repository.clone())),
        get_tag: Arc::new(GetTagHandler::new(tag_repository.clone())),
        create_tag: Arc::new(CreateTagHandler::new(tag_repository.clone())),
        update_tag: Arc::new(UpdateTagHandler::new(tag_repository.clone())),
        delete_tag: Arc::new(DeleteTagHandler::new(tag_repository)),

        config: ApiConfig {
            server: ApiServerConfig {
                base_url: app_config.server.base_url,
            },
            oidc: app_config.oidc.map(|e| ApiOidcConfig {
                sign_in_text: e.sign_in_text,
            }),
            s3: ApiS3Config {
                image_base_url: app_config.s3.image_base_url,
            },
        },
    };

    if let Some(client) = oidc_client
        && let Some(config) = oidc_config
    {
        api_state.build_authorization_url = Some(Arc::new(BuildAuthorizationUrlHandler::new(
            client.clone(),
            config,
        )));
        api_state.exchange_code = Some(Arc::new(ExchangeCodeHandler::new(
            user_repository,
            client,
            jwt_manager,
            jwt_config,
        )));
    }

    let mut api = colette_api::create_router(api_state, app_config.cors.map(|e| e.origin_urls));

    api = api.fallback_service(ServeEmbed::<Asset>::with_parameters(
        Some(String::from("index.html")),
        FallbackBehavior::Ok,
        None,
    ));

    let listener = TcpListener::bind(format!("0.0.0.0:{}", app_config.server.port)).await?;
    let server = axum::serve(
        listener,
        api.into_make_service_with_connect_info::<SocketAddr>(),
    );

    let mut scrape_feed_worker = JobWorker::new(
        scrape_feed_consumer,
        ServiceBuilder::new()
            .concurrency_limit(5)
            .service(ScrapeFeedJobHandler::new(refresh_feed_handler))
            .boxed(),
    );
    let mut scrape_bookmark_worker = JobWorker::new(
        scrape_bookmark_consumer,
        ServiceBuilder::new()
            .concurrency_limit(5)
            .service(ScrapeBookmarkJobHandler::new(refresh_bookmark_handler))
            .boxed(),
    );
    let mut archive_thumbnail_worker = JobWorker::new(
        archive_thumbnail_consumer,
        ServiceBuilder::new()
            .concurrency_limit(5)
            .service(ArchiveThumbnailJobHandler::new(archive_thumbnail_handler))
            .boxed(),
    );
    let mut import_bookmarks_worker = JobWorker::new(
        import_bookmarks_consumer,
        ServiceBuilder::new()
            .service(ImportBookmarksJobHandler::new(
                list_bookmarks_handler,
                Arc::new(Mutex::new(scrape_bookmark_producer)),
            ))
            .boxed(),
    );

    let start_refresh_feeds_worker = async {
        let mut worker = CronWorker::new(
            "refresh_feeds",
            "0 * * * * *".parse().unwrap(),
            ServiceBuilder::new()
                .service(RefreshFeedsJobHandler::new(
                    list_feeds_handler,
                    Arc::new(Mutex::new(scrape_feed_producer)),
                ))
                .boxed(),
        );

        worker.start().await;
    };

    let _ = tokio::join!(
        server,
        scrape_feed_worker.start(),
        scrape_bookmark_worker.start(),
        archive_thumbnail_worker.start(),
        import_bookmarks_worker.start(),
        start_refresh_feeds_worker
    );

    Ok(())
}
