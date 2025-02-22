use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::folder;
use uuid::Uuid;

use super::{FOLDERS_TAG, Folder};
use crate::api::{
    ApiState,
    common::{AuthUser, BaseError, Error, NonEmptyString},
};

#[utoipa::path(
  post,
  path = "",
  request_body = FolderCreate,
  responses(CreateResponse),
  operation_id = "createFolder",
  description = "Create a folder",
  tag = FOLDERS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<FolderCreate>,
) -> Result<impl IntoResponse, Error> {
    match state
        .folder_service
        .create_folder(body.into(), user_id)
        .await
    {
        Ok(data) => Ok(CreateResponse::Created(data.into())),
        Err(e) => match e {
            folder::Error::Conflict(_) => Ok(CreateResponse::Conflict(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FolderCreate {
    #[schema(value_type = String, min_length = 1)]
    pub title: NonEmptyString,
    pub parent_id: Option<Uuid>,
}

impl From<FolderCreate> for folder::FolderCreate {
    fn from(value: FolderCreate) -> Self {
        Self {
            title: value.title.into(),
            parent_id: value.parent_id,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created folder")]
    Created(Folder),

    #[response(status = 409, description = "Folder already exists")]
    Conflict(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
