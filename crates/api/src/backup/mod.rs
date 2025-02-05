use std::sync::Arc;

use colette_core::backup::BackupService;
use colette_task::{import_bookmarks, import_feeds, Storage};
use tokio::sync::Mutex;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

mod export_netscape;
mod export_opml;
mod import_netscape;
mod import_opml;

#[derive(Clone, axum::extract::FromRef)]
pub struct BackupState {
    backup_service: Arc<BackupService>,
    import_feeds_storage: Arc<Mutex<dyn Storage<Job = import_feeds::Job>>>,
    import_bookmarks_storage: Arc<Mutex<dyn Storage<Job = import_bookmarks::Job>>>,
}

impl BackupState {
    pub fn new(
        backup_service: Arc<BackupService>,
        import_feeds_storage: Arc<Mutex<dyn Storage<Job = import_feeds::Job>>>,
        import_bookmarks_storage: Arc<Mutex<dyn Storage<Job = import_bookmarks::Job>>>,
    ) -> Self {
        Self {
            backup_service,
            import_feeds_storage,
            import_bookmarks_storage,
        }
    }
}

#[derive(OpenApi)]
pub struct BackupApi;

impl BackupApi {
    pub fn router() -> OpenApiRouter<BackupState> {
        OpenApiRouter::with_openapi(BackupApi::openapi())
            .routes(routes!(import_opml::handler))
            .routes(routes!(export_opml::handler))
            .routes(routes!(import_netscape::handler))
            .routes(routes!(export_netscape::handler))
    }
}
