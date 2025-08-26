use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_crud::EntryError;
use colette_handler::{Handler as _, MarkEntryAsReadCommand, MarkEntryAsReadError};
use uuid::Uuid;

use crate::api::{
    ApiState,
    common::{ApiError, Auth, Id, Path},
    entry::ENTRIES_TAG,
};

#[utoipa::path(
  post,
  path = "/{id}/markAsRead",
  params(Id),
  responses(OkResponse, ErrResponse),
  operation_id = "markEntryAsRead",
  description = "Mark an entry as read",
  tag = ENTRIES_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state
        .mark_entry_as_read
        .handle(MarkEntryAsReadCommand {
            id: id.into(),
            user_id,
        })
        .await
    {
        Ok(()) => Ok(OkResponse),
        Err(e) => match e {
            MarkEntryAsReadError::Entry(EntryError::NotFound(_)) => {
                Err(ErrResponse::NotFound(e.into()))
            }
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::NO_CONTENT, description = "Successfully marked entry as read")]
pub(super) struct OkResponse;

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::NOT_FOUND, description = "Entry not found")]
    NotFound(ApiError),

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
