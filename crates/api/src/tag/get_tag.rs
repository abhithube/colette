use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::tag::{self};

use super::{TAGS_TAG, TagDetails};
use crate::{
    ApiState,
    common::{AuthUser, BaseError, Error, Id},
};

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id, TagGetQuery),
    responses(GetResponse),
    operation_id = "getTag",
    description = "Get a tag by ID",
    tag = TAGS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Query(query): Query<TagGetQuery>,
    AuthUser(user_id): AuthUser,
) -> Result<GetResponse, Error> {
    match state
        .tag_service
        .get_tag(
            tag::TagGetQuery {
                id,
                with_feed_count: query.with_feed_count,
                with_bookmark_count: query.with_bookmark_count,
            },
            user_id,
        )
        .await
    {
        Ok(data) => Ok(GetResponse::Ok(data.into())),
        Err(e) => match e {
            tag::Error::Forbidden(_) => Ok(GetResponse::Forbidden(BaseError {
                message: e.to_string(),
            })),
            tag::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct TagGetQuery {
    #[serde(default = "with_feed_count")]
    pub with_feed_count: bool,
    #[serde(default = "with_bookmark_count")]
    pub with_bookmark_count: bool,
}

fn with_feed_count() -> bool {
    false
}

fn with_bookmark_count() -> bool {
    false
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum GetResponse {
    #[response(status = 200, description = "Tag by ID")]
    Ok(TagDetails),

    #[response(status = 403, description = "User not authorized")]
    Forbidden(BaseError),

    #[response(status = 404, description = "Tag not found")]
    NotFound(BaseError),
}

impl IntoResponse for GetResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}
