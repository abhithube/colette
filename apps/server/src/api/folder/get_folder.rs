use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::folder::{self, FolderService};

use super::Folder;
use crate::api::common::{BaseError, Error, FOLDERS_TAG, Id, Session};

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getFolder",
    description = "Get a folder by ID",
    tag = FOLDERS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(service): State<Arc<FolderService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service.get_folder(id, session.user_id).await {
        Ok(data) => Ok(GetResponse::Ok(data.into())),
        Err(e) => match e {
            folder::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum GetResponse {
    #[response(status = 200, description = "Folder by ID")]
    Ok(Folder),

    #[response(status = 404, description = "Folder not found")]
    NotFound(BaseError),
}

impl IntoResponse for GetResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}
