use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::tag::{self};

use super::{TAGS_TAG, TagDetails};
use crate::{
    ApiState,
    common::{ApiError, Auth, Id, Path, Query},
};

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id, TagGetQuery),
    responses(OkResponse, ErrResponse),
    operation_id = "getTag",
    description = "Get a tag by ID",
    tag = TAGS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Query(query): Query<TagGetQuery>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state
        .tag_service
        .get_tag(
            tag::TagGetQuery {
                id,
                with_subscription_count: query.with_subscription_count,
                with_bookmark_count: query.with_bookmark_count,
            },
            user_id,
        )
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => match e {
            tag::Error::Forbidden(_) => Err(ErrResponse::Forbidden(e.into())),
            tag::Error::NotFound(_) => Err(ErrResponse::NotFound(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub(super) struct TagGetQuery {
    /// Whether to include the count of subscriptions the tag is linked to
    #[serde(default = "with_subscription_count")]
    with_subscription_count: bool,
    /// Whether to include the count of bookmarks the tag is linked to
    #[serde(default = "with_bookmark_count")]
    with_bookmark_count: bool,
}

fn with_subscription_count() -> bool {
    false
}

fn with_bookmark_count() -> bool {
    false
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Tag by ID")]
pub(super) struct OkResponse(TagDetails);

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

    #[response(status = StatusCode::FORBIDDEN, description = "User not authorized")]
    Forbidden(ApiError),

    #[response(status = StatusCode::NOT_FOUND, description = "Tag not found")]
    NotFound(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
