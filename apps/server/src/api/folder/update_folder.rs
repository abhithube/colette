use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::folder;
use uuid::Uuid;

use super::{FOLDERS_TAG, Folder};
use crate::api::{
    ApiState,
    common::{AuthUser, BaseError, Error, Id, NonEmptyString},
};

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = FolderUpdate,
    responses(UpdateResponse),
    operation_id = "updateFolder",
    description = "Update a folder by ID",
    tag = FOLDERS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<FolderUpdate>,
) -> Result<impl IntoResponse, Error> {
    match state
        .folder_service
        .update_folder(id, body.into(), user_id)
        .await
    {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
        Err(e) => match e {
            folder::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FolderUpdate {
    #[schema(value_type = Option<String>, min_length = 1, nullable = false)]
    pub title: Option<NonEmptyString>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub parent_id: Option<Option<Uuid>>,
}

impl From<FolderUpdate> for folder::FolderUpdate {
    fn from(value: FolderUpdate) -> Self {
        Self {
            title: value.title.map(Into::into),
            parent_id: value.parent_id,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated folder")]
    Ok(Folder),

    #[response(status = 404, description = "Folder not found")]
    NotFound(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for UpdateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
