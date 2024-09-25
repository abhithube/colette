use std::{fs, str::FromStr, sync::Arc};

use colette_api::Session;
use colette_core::{
    auth::{AuthService, Login, Register},
    common::{Findable, NonEmptyString},
    feed::FeedService,
    feed_entry::FeedEntryService,
    profile::{ProfileIdOrDefaultParams, ProfileService},
};
use colette_migration::{Migrator, MigratorTrait};
use colette_plugins::register_feed_plugins;
use colette_repository::{
    FeedEntrySqlRepository, FeedSqlRepository, ProfileSqlRepository, UserSqlRepository,
};
use colette_util::{base64::Base64Encoder, password::ArgonHasher};
use command::{auth, feed, feed_entry, profile};
use email_address::EmailAddress;
use sea_orm::{ConnectOptions, Database};
use tauri::Manager;

mod command;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            tauri::async_runtime::block_on(async move {
                let mut path = app.path().app_data_dir()?;
                if !path.exists() {
                    fs::create_dir_all(&path)?;
                }
                path = path.join("sqlite.db");

                let database_url = format!("sqlite://{}?mode=rwc", path.to_string_lossy());

                let mut opts = ConnectOptions::new(database_url);
                opts.max_connections(100);

                let db = Database::connect(opts).await?;
                Migrator::up(&db, None).await?;

                let profile_repository = Arc::new(ProfileSqlRepository::new(db.clone()));

                let auth_service = AuthService::new(
                    Arc::new(UserSqlRepository::new(db.clone())),
                    profile_repository.clone(),
                    Arc::new(ArgonHasher),
                );
                let feed_service = FeedService::new(
                    Arc::new(FeedSqlRepository::new(db.clone())),
                    Arc::new(register_feed_plugins()),
                );
                let feed_entry_service = Arc::new(FeedEntryService::new(
                    Arc::new(FeedEntrySqlRepository::new(db)),
                    Arc::new(Base64Encoder),
                ));
                let profile_service = ProfileService::new(profile_repository.clone());

                let email = EmailAddress::from_str("default@default.com")?;
                let password = NonEmptyString::try_from("default".to_owned())?;
                let profile = match auth_service
                    .login(Login {
                        email: email.clone(),
                        password: password.clone(),
                    })
                    .await
                {
                    Ok(profile) => profile,
                    _ => {
                        let user = auth_service.register(Register { email, password }).await?;
                        profile_repository
                            .find(ProfileIdOrDefaultParams {
                                user_id: user.id,
                                ..Default::default()
                            })
                            .await?
                    }
                };

                app.manage(Session {
                    profile_id: profile.id,
                    user_id: profile.user_id,
                });

                app.manage(auth_service);
                app.manage(feed_service);
                app.manage(feed_entry_service);
                app.manage(profile_service);

                Ok(())
            })
        })
        .invoke_handler(tauri::generate_handler![
            auth::register,
            auth::login,
            auth::get_active_user,
            auth::switch_profile
        ])
        .invoke_handler(tauri::generate_handler![
            feed::list_feeds,
            feed::create_feed,
            feed::get_feed,
            feed::update_feed,
            feed::delete_feed
        ])
        .invoke_handler(tauri::generate_handler![
            feed_entry::list_feed_entries,
            feed_entry::get_feed_entry,
            feed_entry::update_feed_entry
        ])
        .invoke_handler(tauri::generate_handler![
            profile::list_profiles,
            profile::create_profile,
            profile::get_profile,
            profile::get_active_profile,
            profile::update_profile,
            profile::delete_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
