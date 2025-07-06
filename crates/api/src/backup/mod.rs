use axum::{Router, routing};
use utoipa::OpenApi;

use super::ApiState;

mod export_backup;
mod import_backup;

const BACKUPS_TAG: &str = "Backups";

#[derive(OpenApi)]
#[openapi(paths(import_backup::handler, export_backup::handler))]
pub(crate) struct BackupApi;

impl BackupApi {
    pub(crate) fn router() -> Router<ApiState> {
        Router::new()
            .route("/import", routing::post(import_backup::handler))
            .route("/export", routing::post(export_backup::handler))
    }
}
