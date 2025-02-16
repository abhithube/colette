use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
};

use super::BACKUPS_TAG;
use crate::api::{
    ApiState,
    common::{AuthUser, Error},
};

#[utoipa::path(
  post,
  path = "/opml/export",
  responses(ExportOpmlResponse),
  operation_id = "exportOpml",
  description = "Export OPML feeds",
  tag = BACKUPS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    AuthUser(user_id): AuthUser,
) -> Result<ExportOpmlResponse, Error> {
    match state.backup_service.export_opml(user_id).await {
        Ok(data) => Ok(ExportOpmlResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, utoipa::IntoResponses)]
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
