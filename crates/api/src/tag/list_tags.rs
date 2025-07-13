use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::tag::{self, TagCursor};

use super::{TAGS_TAG, TagDetails};
use crate::{
    ApiState,
    common::{ApiError, Auth, Query},
    pagination::{PAGINATION_LIMIT, Paginated, decode_cursor},
};

#[utoipa::path(
    get,
    path = "",
    params(TagListQuery),
    responses(OkResponse, ErrResponse),
    operation_id = "listTags",
    description = "List user tags",
    tag = TAGS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<TagListQuery>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state
        .tag_service
        .list_tags(query.try_into()?, user_id)
        .await
    {
        Ok(tags) => {
            let data = tags.try_into().map_err(ErrResponse::InternalServerError)?;

            Ok(OkResponse(data))
        }
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub(super) struct TagListQuery {
    /// Filter by the type of tag
    #[param(inline)]
    tag_type: Option<TagType>,
    /// Pagination cursor
    #[param(nullable = false)]
    cursor: Option<String>,
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

impl TryFrom<TagListQuery> for tag::TagListQuery {
    type Error = ErrResponse;

    fn try_from(value: TagListQuery) -> Result<Self, Self::Error> {
        let cursor = value
            .cursor
            .map(|e| decode_cursor::<TagCursor>(&e))
            .transpose()
            .map_err(|e| ErrResponse::InternalServerError(e.into()))?;

        Ok(Self {
            tag_type: value.tag_type.map(Into::into),
            cursor,
            limit: Some(PAGINATION_LIMIT),
            with_subscription_count: value.with_subscription_count,
            with_bookmark_count: value.with_bookmark_count,
        })
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) enum TagType {
    Bookmarks,
    Feeds,
}

impl From<TagType> for tag::TagType {
    fn from(value: TagType) -> Self {
        match value {
            TagType::Bookmarks => Self::Bookmarks,
            TagType::Feeds => Self::Feeds,
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Paginated list of tags")]
pub(super) struct OkResponse(Paginated<TagDetails>);

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
