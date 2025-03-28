use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use colette_core::tag;

use super::{TAGS_TAG, Tag};
use crate::{
    ApiState,
    common::{AuthUser, Error, Paginated},
};

#[utoipa::path(
    get,
    path = "",
    params(TagListQuery),
    responses(ListResponse),
    operation_id = "listTags",
    description = "List user tags",
    tag = TAGS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<TagListQuery>,
    AuthUser(user_id): AuthUser,
) -> Result<ListResponse, Error> {
    match state.tag_service.list_tags(query.into(), user_id).await {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct TagListQuery {
    #[param(inline)]
    #[serde(default = "TagType::default")]
    pub tag_type: TagType,
}

impl From<TagListQuery> for tag::TagListQuery {
    fn from(value: TagListQuery) -> Self {
        Self {
            tag_type: value.tag_type.into(),
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum TagType {
    #[default]
    All,
    Bookmarks,
    Feeds,
}

impl From<TagType> for tag::TagType {
    fn from(value: TagType) -> Self {
        match value {
            TagType::All => Self::All,
            TagType::Bookmarks => Self::Bookmarks,
            TagType::Feeds => Self::Feeds,
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of tags")]
    Ok(Paginated<Tag>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
