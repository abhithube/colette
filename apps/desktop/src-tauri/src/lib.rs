use std::{fs, str::FromStr};

use colette_api::Session;
use colette_backup::{netscape::NetscapeManager, opml::OpmlManager};
use colette_core::{
    auth::{AuthService, Login, Register},
    backup::BackupService,
    bookmark::BookmarkService,
    common::NonEmptyString,
    feed::FeedService,
    feed_entry::FeedEntryService,
    scraper::ScraperService,
    smart_feed::SmartFeedService,
    tag::TagService,
};
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_queue::memory::InMemoryQueue;
use colette_repository::sqlite::{
    SqliteBackupRepository, SqliteBookmarkRepository, SqliteFeedEntryRepository,
    SqliteFeedRepository, SqliteScraperRepository, SqliteSmartFeedRepository, SqliteTagRepository,
    SqliteUserRepository,
};
use colette_scraper::{
    bookmark::DefaultBookmarkScraper, downloader::DefaultDownloader, feed::DefaultFeedScraper,
};
use colette_task::{import_bookmarks, import_feeds, scrape_bookmark, scrape_feed};
use colette_util::{base64::Base64Encoder, password::ArgonHasher};
use colette_worker::run_task_worker;
use command::{auth, backup, bookmark, feed, feed_entry, smart_feed, tag};
use deadpool_sqlite::{Config, Runtime};
use email_address::EmailAddress;
use refinery::embed_migrations;
use tauri::Manager;
use tower::ServiceBuilder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod command;

embed_migrations!("../migrations");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            tauri::async_runtime::block_on(async move {
                let mut path = app.path().app_data_dir()?;
                if !path.exists() {
                    fs::create_dir_all(&path)?;
                }
                path = path.join("sqlite.db");

                let pool = Config::new(path).create_pool(Runtime::Tokio1)?;

                let conn = pool.get().await?;
                conn.interact(move |conn| migrations::runner().run(conn))
                    .await
                    .unwrap()?;

                let backup_repository = Box::new(SqliteBackupRepository::new(pool.clone()));
                let bookmark_repository = Box::new(SqliteBookmarkRepository::new(pool.clone()));
                let feed_repository = Box::new(SqliteFeedRepository::new(pool.clone()));
                let feed_entry_repository = Box::new(SqliteFeedEntryRepository::new(pool.clone()));
                let scraper_repository = Box::new(SqliteScraperRepository::new(pool.clone()));
                let smart_feed_repository = Box::new(SqliteSmartFeedRepository::new(pool.clone()));
                let tag_repository = Box::new(SqliteTagRepository::new(pool.clone()));
                let user_repository = Box::new(SqliteUserRepository::new(pool));

                let client = colette_http::Client::build(None, None)?;
                let downloader = Box::new(DefaultDownloader::new(client.clone()));
                let feed_scraper = Box::new(DefaultFeedScraper::new(downloader.clone()));
                let bookmark_scraper = Box::new(DefaultBookmarkScraper::new(downloader.clone()));
                let feed_plugin_registry =
                    Box::new(register_feed_plugins(downloader.clone(), feed_scraper));
                let bookmark_plugin_registry =
                    Box::new(register_bookmark_plugins(client, bookmark_scraper));

                let base64_encoder = Box::new(Base64Encoder);

                let auth_service = AuthService::new(user_repository, Box::new(ArgonHasher));
                let backup_service = BackupService::new(
                    backup_repository,
                    feed_repository.clone(),
                    bookmark_repository.clone(),
                    Box::new(OpmlManager),
                    Box::new(NetscapeManager),
                );
                let bookmark_service = BookmarkService::new(
                    bookmark_repository,
                    bookmark_plugin_registry.clone(),
                    base64_encoder.clone(),
                );
                let feed_service = FeedService::new(feed_repository, feed_plugin_registry.clone());
                let feed_entry_service =
                    FeedEntryService::new(feed_entry_repository, base64_encoder);
                let scraper_service = ScraperService::new(
                    scraper_repository,
                    feed_plugin_registry,
                    bookmark_plugin_registry,
                );
                let smart_feed_service = SmartFeedService::new(smart_feed_repository);
                let tag_service = TagService::new(tag_repository);

                let (scrape_feed_queue, scrape_feed_receiver) = InMemoryQueue::new();
                let (scrape_bookmark_queue, scrape_bookmark_receiver) = InMemoryQueue::new();
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
                let import_feeds_task = import_feeds::Task::new(scrape_feed_queue);
                let import_bookmarks_task = import_bookmarks::Task::new(scrape_bookmark_queue);

                let email = EmailAddress::from_str("default@default.com")?;
                let password = NonEmptyString::try_from("default".to_owned())?;
                let user = match auth_service
                    .login(Login {
                        email: email.clone(),
                        password: password.clone(),
                    })
                    .await
                {
                    Ok(user) => Ok(user),
                    _ => auth_service.register(Register { email, password }).await,
                }?;

                app.manage(Session { user_id: user.id });

                app.manage(auth_service);
                app.manage(backup_service);
                app.manage(bookmark_service);
                app.manage(feed_service);
                app.manage(feed_entry_service);
                app.manage(smart_feed_service);
                app.manage(tag_service);

                app.manage(import_feeds_queue);
                app.manage(import_bookmarks_queue);

                tokio::spawn(run_task_worker(scrape_feed_receiver, scrape_feed_task));
                tokio::spawn(run_task_worker(
                    scrape_bookmark_receiver,
                    scrape_bookmark_task,
                ));
                tokio::spawn(run_task_worker(import_feeds_receiver, import_feeds_task));
                tokio::spawn(run_task_worker(
                    import_bookmarks_receiver,
                    import_bookmarks_task,
                ));

                Ok(())
            })
        })
        .invoke_handler(tauri::generate_handler![
            auth::register,
            auth::login,
            auth::get_active_user,
            backup::import_opml,
            backup::export_opml,
            backup::import_netscape,
            backup::export_netscape,
            bookmark::list_bookmarks,
            bookmark::create_bookmark,
            bookmark::get_bookmark,
            bookmark::update_bookmark,
            bookmark::delete_bookmark,
            bookmark::scrape_bookmark,
            feed::list_feeds,
            feed::create_feed,
            feed::get_feed,
            feed::update_feed,
            feed::delete_feed,
            feed::detect_feeds,
            feed_entry::list_feed_entries,
            feed_entry::get_feed_entry,
            feed_entry::update_feed_entry,
            smart_feed::list_smart_feeds,
            smart_feed::create_smart_feed,
            smart_feed::get_smart_feed,
            smart_feed::update_smart_feed,
            smart_feed::delete_smart_feed,
            tag::list_tags,
            tag::create_tag,
            tag::get_tag,
            tag::update_tag,
            tag::delete_tag,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
