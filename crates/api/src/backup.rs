use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use bytes::{Buf, Bytes, BytesMut};
use colette_core::backup::BackupService;
use http::{HeaderMap, HeaderValue, StatusCode};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::common::{Error, Session, BACKUPS_TAG};

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
            .routes(routes!(import_opml))
            .routes(routes!(export_opml))
    }
}

#[utoipa::path(
    post,
    path = "/opml/import",
    request_body = Vec<u8>,
    responses(ImportResponse),
    operation_id = "importOpml",
    description = "Import OPML feeds into profile",
    tag = BACKUPS_TAG
)]
#[axum::debug_handler]
pub async fn import_opml(
    State(service): State<Arc<BackupService>>,
    session: Session,
    bytes: Bytes,
) -> Result<ImportResponse, Error> {
    match service
        .import_opml(bytes.reader(), session.profile_id)
        .await
    {
        Ok(_) => Ok(ImportResponse::NoContent),
        _ => Err(Error::Unknown),
    }
}

#[utoipa::path(
    post,
    path = "/opml/export",
    responses(ExportResponse),
    operation_id = "exportOpml",
    description = "Export OPML feeds from profile",
    tag = BACKUPS_TAG
)]
#[axum::debug_handler]
pub async fn export_opml(
    State(service): State<Arc<BackupService>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let mut buffer = BytesMut::new();
    match service.export_opml(&mut buffer, session.profile_id).await {
        Ok(_) => Ok(ExportResponse::Ok(buffer.into())),
        _ => Err(Error::Unknown),
    }
}

#[derive(Debug, utoipa::IntoResponses)]
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

#[derive(Debug, utoipa::IntoResponses)]
pub enum ExportResponse {
    #[response(
        status = 200,
        description = "OPML file",
        content_type = "application/xml"
    )]
    Ok(Vec<u8>),
}

impl IntoResponse for ExportResponse {
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
