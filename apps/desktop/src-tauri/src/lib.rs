use std::fs;

use colette_migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database};
use tauri::Manager;

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

                Ok(())
            })
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
