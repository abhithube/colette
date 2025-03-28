use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::ApiState;

mod export_netscape;
mod export_opml;
mod import_netscape;
mod import_opml;

pub const BACKUPS_TAG: &str = "Backups";

#[derive(OpenApi)]
pub struct BackupApi;

impl BackupApi {
    pub fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(BackupApi::openapi())
            .routes(routes!(import_opml::handler))
            .routes(routes!(export_opml::handler))
            .routes(routes!(import_netscape::handler))
            .routes(routes!(export_netscape::handler))
    }
}
