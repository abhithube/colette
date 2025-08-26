use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_handler::{Handler as _, ListPatsQuery, PatCursor};

use crate::api::{
    ApiState,
    common::{ApiError, Auth, Query},
    pagination::{PAGINATION_LIMIT, Paginated, decode_cursor},
    pat::{PERSONAL_ACCESS_TOKENS_TAG, PersonalAccessToken},
};

#[utoipa::path(
    get,
    path = "",
    params(PatListQuery),
    responses(OkResponse, ErrResponse),
    operation_id = "listPats",
    description = "List user PATs",
    tag = PERSONAL_ACCESS_TOKENS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<PatListQuery>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    let cursor = query
        .cursor
        .map(|e| decode_cursor::<PatCursor>(&e))
        .transpose()
        .map_err(|e| ErrResponse::InternalServerError(e.into()))?;

    match state
        .list_pats
        .handle(ListPatsQuery {
            cursor,
            limit: Some(PAGINATION_LIMIT),
            user_id: user_id.as_inner(),
        })
        .await
    {
        Ok(pats) => {
            let data = pats.try_into().map_err(ErrResponse::InternalServerError)?;

            Ok(OkResponse(data))
        }
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub(super) struct PatListQuery {
    /// Pagination cursor
    #[param(nullable = false)]
    cursor: Option<String>,
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Paginated list of PATs")]
pub(super) struct OkResponse(Paginated<PersonalAccessToken>);

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

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
