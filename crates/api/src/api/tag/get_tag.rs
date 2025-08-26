use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_crud::TagError;
use colette_handler::{GetTagError, GetTagQuery, Handler as _};

use crate::api::{
    ApiState,
    common::{ApiError, Auth, Id, Path},
    tag::{TAGS_TAG, Tag},
};

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(OkResponse, ErrResponse),
    operation_id = "getTag",
    description = "Get a tag by ID",
    tag = TAGS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state
        .get_tag
        .handle(GetTagQuery {
            id,
            user_id: user_id.as_inner(),
        })
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => match e {
            GetTagError::Tag(TagError::NotFound(_)) => Err(ErrResponse::NotFound(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Tag by ID")]
pub(super) struct OkResponse(Tag);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::NOT_FOUND, description = "Tag not found")]
    NotFound(ApiError),

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
        }
    }
}
