use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_valid::Valid;
use colette_core::tag::{self, TagCreateData, TagFindManyFilters, TagRepository, TagUpdateData};
use uuid::Uuid;

use crate::common::{BaseError, Error, Id, Paginated, Session, TagList};

#[derive(Clone, axum::extract::FromRef)]
pub struct TagState {
    pub repository: Arc<dyn TagRepository>,
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(list_tags, get_tag, create_tag, update_tag, delete_tag),
    components(schemas(Tag, TagList, TagCreate, TagUpdate))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<TagState> {
        Router::new().nest(
            "/tags",
            Router::new()
                .route("/", routing::get(list_tags).post(create_tag))
                .route(
                    "/:id",
                    routing::get(get_tag).patch(update_tag).delete(delete_tag),
                ),
        )
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
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

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct TagCreate {
    #[schema(min_length = 1)]
    #[validate(length(min = 1, message = "cannot be empty"))]
    pub title: String,
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct TagUpdate {
    #[schema(min_length = 1, nullable = false)]
    #[validate(length(min = 1, message = "cannot be empty"))]
    pub title: Option<String>,
}

impl From<TagUpdate> for TagUpdateData {
    fn from(value: TagUpdate) -> Self {
        Self { title: value.title }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::IntoParams, validator::Validate)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct ListTagsQuery {
    #[param(inline)]
    #[serde(default = "TagType::default")]
    pub tag_type: TagType,
}

impl From<ListTagsQuery> for TagFindManyFilters {
    fn from(value: ListTagsQuery) -> Self {
        Self {
            tag_type: value.tag_type.into(),
        }
    }
}

#[derive(Clone, Debug, Default, serde::Deserialize, utoipa::ToSchema)]
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
    params(ListTagsQuery),
    responses(ListResponse),
    operation_id = "listTags",
    description = "List the active profile tags"
)]
#[axum::debug_handler]
pub async fn list_tags(
    State(repository): State<Arc<dyn TagRepository>>,
    Valid(Query(query)): Valid<Query<ListTagsQuery>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .find_many_tags(session.profile_id, None, None, Some(query.into()))
        .await
        .map(Paginated::<Tag>::from);

    match result {
        Ok(data) => Ok(ListResponse::Ok(data)),
        _ => Err(Error::Unknown),
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getTag",
    description = "Get a tag by ID"
)]
#[axum::debug_handler]
pub async fn get_tag(
    State(repository): State<Arc<dyn TagRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .find_one_tag(id, session.profile_id)
        .await
        .map(Tag::from);

    match result {
        Ok(data) => Ok(GetResponse::Ok(data)),
        Err(e) => match e {
            tag::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[utoipa::path(
  post,
  path = "",
  request_body = TagCreate,
  responses(CreateResponse),
  operation_id = "createTag",
  description = "Create a tag"
)]
#[axum::debug_handler]
pub async fn create_tag(
    State(repository): State<Arc<dyn TagRepository>>,
    session: Session,
    Valid(Json(body)): Valid<Json<TagCreate>>,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .create_tag(TagCreateData {
            title: body.title,
            profile_id: session.profile_id,
        })
        .await
        .map(Tag::from);

    match result {
        Ok(data) => Ok(CreateResponse::Created(data)),
        Err(e) => match e {
            tag::Error::Conflict(_) => Ok(CreateResponse::Conflict(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
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
    description = "Update a tag by ID"
)]
#[axum::debug_handler]
pub async fn update_tag(
    State(repository): State<Arc<dyn TagRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Valid(Json(body)): Valid<Json<TagUpdate>>,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .update_tag(id, session.profile_id, body.into())
        .await
        .map(Tag::from);

    match result {
        Ok(data) => Ok(UpdateResponse::Ok(data)),
        Err(e) => match e {
            tag::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[utoipa::path(
    delete,
    path = "/{id}",
    params(Id),
    responses(DeleteResponse),
    operation_id = "deleteTag",
    description = "Delete a tag by ID"
)]
#[axum::debug_handler]
pub async fn delete_tag(
    State(repository): State<Arc<dyn TagRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository.delete_tag(id, session.profile_id).await;

    match result {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            tag::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of tags")]
    Ok(TagList),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
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

#[derive(Debug, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created tag")]
    Created(Tag),

    #[response(status = 409, description = "Tag already exists")]
    Conflict(BaseError),

    #[allow(dead_code)]
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

#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated tag")]
    Ok(Tag),

    #[response(status = 404, description = "Tag not found")]
    NotFound(BaseError),

    #[allow(dead_code)]
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

#[derive(Debug, utoipa::IntoResponses)]
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
