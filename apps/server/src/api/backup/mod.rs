use std::sync::Arc;

use colette_core::backup::BackupService;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

mod export_netscape;
mod export_opml;
mod import_netscape;
mod import_opml;

#[derive(Clone, axum::extract::FromRef)]
pub struct BackupState {
    backup_service: Arc<BackupService>,
}

impl BackupState {
    pub fn new(backup_service: Arc<BackupService>) -> Self {
        Self { backup_service }
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
