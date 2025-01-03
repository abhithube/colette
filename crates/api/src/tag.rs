use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use colette_core::{
    common::NonEmptyString,
    tag::{self, TagService},
};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    common::{BaseError, Error, Id, Session, TAGS_TAG},
    Paginated,
};

#[derive(Clone, axum::extract::FromRef)]
pub struct TagState {
    service: Arc<TagService>,
}

impl TagState {
    pub fn new(service: Arc<TagService>) -> Self {
        Self { service }
    }
}

#[derive(OpenApi)]
#[openapi(components(schemas(Tag, Paginated<Tag>, TagCreate, TagUpdate)))]
pub struct TagApi;

impl TagApi {
    pub fn router() -> OpenApiRouter<TagState> {
        OpenApiRouter::with_openapi(TagApi::openapi())
            .routes(routes!(list_tags, create_tag))
            .routes(routes!(get_tag, update_tag, delete_tag))
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: Uuid,
    pub title: String,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    bookmark_count: Option<i64>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed_count: Option<i64>,
}

impl From<colette_core::Tag> for Tag {
    fn from(value: colette_core::Tag) -> Self {
        Self {
            id: value.id,
            title: value.title,
            bookmark_count: value.bookmark_count,
            feed_count: value.feed_count,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TagCreate {
    #[schema(value_type = String, min_length = 1)]
    pub title: NonEmptyString,
}

impl From<TagCreate> for tag::TagCreate {
    fn from(value: TagCreate) -> Self {
        Self { title: value.title }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TagUpdate {
    #[schema(value_type = Option<String>, min_length = 1, nullable = false)]
    pub title: Option<NonEmptyString>,
}

impl From<TagUpdate> for tag::TagUpdate {
    fn from(value: TagUpdate) -> Self {
        Self { title: value.title }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::IntoParams)]
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

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
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
pub async fn list_tags(
    State(service): State<Arc<TagService>>,
    Query(query): Query<TagListQuery>,
    session: Session,
) -> Result<ListResponse, Error> {
    match service.list_tags(query.into(), session.user_id).await {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getTag",
    description = "Get a tag by ID",
    tag = TAGS_TAG
)]
#[axum::debug_handler]
pub async fn get_tag(
    State(service): State<Arc<TagService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<GetResponse, Error> {
    match service.get_tag(id, session.user_id).await {
        Ok(data) => Ok(GetResponse::Ok(data.into())),
        Err(e) => match e {
            tag::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[utoipa::path(
  post,
  path = "",
  request_body = TagCreate,
  responses(CreateResponse),
  operation_id = "createTag",
  description = "Create a tag",
  tag = TAGS_TAG
)]
#[axum::debug_handler]
pub async fn create_tag(
    State(service): State<Arc<TagService>>,
    session: Session,
    Json(body): Json<TagCreate>,
) -> Result<CreateResponse, Error> {
    match service.create_tag(body.into(), session.user_id).await {
        Ok(data) => Ok(CreateResponse::Created(data.into())),
        Err(e) => match e {
            tag::Error::Conflict(_) => Ok(CreateResponse::Conflict(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = TagUpdate,
    responses(UpdateResponse),
    operation_id = "updateTag",
    description = "Update a tag by ID",
    tag = TAGS_TAG
)]
#[axum::debug_handler]
pub async fn update_tag(
    State(service): State<Arc<TagService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Json(body): Json<TagUpdate>,
) -> Result<UpdateResponse, Error> {
    match service.update_tag(id, body.into(), session.user_id).await {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
        Err(e) => match e {
            tag::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[utoipa::path(
    delete,
    path = "/{id}",
    params(Id),
    responses(DeleteResponse),
    operation_id = "deleteTag",
    description = "Delete a tag by ID",
    tag = TAGS_TAG
)]
#[axum::debug_handler]
pub async fn delete_tag(
    State(service): State<Arc<TagService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<DeleteResponse, Error> {
    match service.delete_tag(id, session.user_id).await {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            tag::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
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

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum GetResponse {
    #[response(status = 200, description = "Tag by ID")]
    Ok(Tag),

    #[response(status = 404, description = "Tag not found")]
    NotFound(BaseError),
}

impl IntoResponse for GetResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created tag")]
    Created(Tag),

    #[response(status = 409, description = "Tag already exists")]
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

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated tag")]
    Ok(Tag),

    #[response(status = 404, description = "Tag not found")]
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

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum DeleteResponse {
    #[response(status = 204, description = "Successfully deleted tag")]
    NoContent,

    #[response(status = 404, description = "Tag not found")]
    NotFound(BaseError),
}

impl IntoResponse for DeleteResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}
