use std::sync::Arc;

use apalis::prelude::Storage;
use apalis_redis::RedisStorage;
use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use colette_core::backup::BackupService;
use colette_task::{import_bookmarks, import_feeds};
use tokio::sync::Mutex;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::common::{Error, Session, BACKUPS_TAG};

#[derive(Clone, axum::extract::FromRef)]
pub struct BackupState {
    backup_service: Arc<BackupService>,
    import_feeds_storage: Arc<Mutex<RedisStorage<import_feeds::Job>>>,
    import_bookmarks_storage: Arc<Mutex<RedisStorage<import_bookmarks::Job>>>,
}

impl BackupState {
    pub fn new(
        backup_service: Arc<BackupService>,
        import_feeds_storage: Arc<Mutex<RedisStorage<import_feeds::Job>>>,
        import_bookmarks_storage: Arc<Mutex<RedisStorage<import_bookmarks::Job>>>,
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
            .routes(routes!(import_opml))
            .routes(routes!(export_opml))
            .routes(routes!(import_netscape))
            .routes(routes!(export_netscape))
    }
}

#[utoipa::path(
    post,
    path = "/opml/import",
    request_body = Vec<u8>,
    responses(ImportResponse),
    operation_id = "importOpml",
    description = "Import OPML feeds",
    tag = BACKUPS_TAG
)]
#[axum::debug_handler]
pub async fn import_opml(
    State(state): State<BackupState>,
    session: Session,
    bytes: Bytes,
) -> Result<ImportResponse, Error> {
    match state
        .backup_service
        .import_opml(bytes, session.user_id)
        .await
    {
        Ok(urls) => {
            let mut storage = state.import_feeds_storage.lock().await;

            storage
                .push(import_feeds::Job::new(urls))
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            Ok(ImportResponse::NoContent)
        }
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[utoipa::path(
    post,
    path = "/opml/export",
    responses(ExportOpmlResponse),
    operation_id = "exportOpml",
    description = "Export OPML feeds",
    tag = BACKUPS_TAG
)]
#[axum::debug_handler]
pub async fn export_opml(
    State(service): State<Arc<BackupService>>,
    session: Session,
) -> Result<ExportOpmlResponse, Error> {
    match service.export_opml(session.user_id).await {
        Ok(data) => Ok(ExportOpmlResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[utoipa::path(
    post,
    path = "/netscape/import",
    request_body = Vec<u8>,
    responses(ImportResponse),
    operation_id = "importNetscape",
    description = "Import Netscape bookmarks",
    tag = BACKUPS_TAG
)]
#[axum::debug_handler]
pub async fn import_netscape(
    State(state): State<BackupState>,
    session: Session,
    bytes: Bytes,
) -> Result<ImportResponse, Error> {
    match state
        .backup_service
        .import_netscape(bytes, session.user_id)
        .await
    {
        Ok(urls) => {
            let mut storage = state.import_bookmarks_storage.lock().await;

            storage
                .push(import_bookmarks::Job::new(urls))
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            Ok(ImportResponse::NoContent)
        }
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[utoipa::path(
    post,
    path = "/netscape/export",
    responses(ExportNetscapeResponse),
    operation_id = "exportNetscape",
    description = "Export Netscape bookmarks",
    tag = BACKUPS_TAG
)]
#[axum::debug_handler]
pub async fn export_netscape(
    State(service): State<Arc<BackupService>>,
    session: Session,
) -> Result<ExportNetscapeResponse, Error> {
    match service.export_netscape(session.user_id).await {
        Ok(data) => Ok(ExportNetscapeResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum ImportResponse {
    #[response(status = 204, description = "Successfully started import")]
    NoContent,
}

impl IntoResponse for ImportResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum ExportOpmlResponse {
    #[response(
        status = 200,
        description = "OPML file",
        content_type = "application/xml"
    )]
    Ok(Vec<u8>),
}

impl IntoResponse for ExportOpmlResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => {
                let mut headers = HeaderMap::new();
                headers.insert("Content-Type", HeaderValue::from_static("application/xml"));

                (headers, data).into_response()
            }
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum ExportNetscapeResponse {
    #[response(
        status = 200,
        description = "Netscape file",
        content_type = "text/html"
    )]
    Ok(Vec<u8>),
}

impl IntoResponse for ExportNetscapeResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => {
                let mut headers = HeaderMap::new();
                headers.insert("Content-Type", HeaderValue::from_static("text/html"));

                (headers, data).into_response()
            }
        }
    }
}
