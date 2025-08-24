use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_authentication::PatError;
use colette_handler::{GetPatError, GetPatQuery, Handler as _};

use crate::{
    ApiState,
    common::{ApiError, Auth, Id, Path},
    pat::{PERSONAL_ACCESS_TOKENS_TAG, PersonalAccessToken},
};

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(OkResponse, ErrResponse),
    operation_id = "getPat",
    description = "Get a PAT by ID",
    tag = PERSONAL_ACCESS_TOKENS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state
        .get_pat
        .handle(GetPatQuery {
            id,
            user_id: user_id.as_inner(),
        })
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => match e {
            GetPatError::Pat(e) => match e {
                PatError::NotFound(_) => Err(ErrResponse::NotFound(e.into())),
                _ => Err(ErrResponse::InternalServerError(e.into())),
            },
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "PAT by ID")]
pub(super) struct OkResponse(PersonalAccessToken);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::NOT_FOUND, description = "PAT not found")]
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
            _ => unreachable!(),
        }
    }
}
