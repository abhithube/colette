use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::Query;
use colette_core::{
    common::{NonEmptyString, NonEmptyVec},
    feed::{self, FeedService},
};
use url::Url;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    common::{BaseError, Error, Id, Session, FEEDS_TAG},
    tag::Tag,
    Paginated,
};

#[derive(Clone, axum::extract::FromRef)]
pub struct FeedState {
    service: Arc<FeedService>,
}

impl FeedState {
    pub fn new(service: Arc<FeedService>) -> Self {
        Self { service }
    }
}

#[derive(OpenApi)]
#[openapi(components(schemas(
    Feed,
    Paginated<Feed>,
    FeedCreate,
    FeedUpdate,
    FeedDetect,
    FeedDetected,
    FeedProcessed,
    DetectedResponse
)))]
pub struct FeedApi;

impl FeedApi {
    pub fn router() -> OpenApiRouter<FeedState> {
        OpenApiRouter::with_openapi(FeedApi::openapi())
            .routes(routes!(list_feeds, create_feed))
            .routes(routes!(get_feed, update_feed, delete_feed))
            .routes(routes!(detect_feeds))
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub id: Uuid,
    #[schema(format = "uri")]
    pub link: String,
    #[schema(required)]
    pub title: Option<String>,
    pub pinned: bool,
    pub original_title: String,
    #[schema(format = "uri", required)]
    pub url: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
    #[schema(nullable = false)]
    pub unread_count: Option<i64>,
}

impl From<colette_core::Feed> for Feed {
    fn from(value: colette_core::Feed) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            pinned: value.pinned,
            original_title: value.original_title,
            url: value.url,
            tags: value.tags.map(|e| e.into_iter().map(Tag::from).collect()),
            unread_count: value.unread_count,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedCreate {
    #[schema(format = "uri")]
    pub url: Url,
    #[schema(value_type = Option<String>, min_length = 1)]
    pub title: Option<NonEmptyString>,
    #[schema(nullable = false, default = false)]
    pub pinned: Option<bool>,
    #[schema(value_type = Option<Vec<String>>, nullable = false, min_length = 1)]
    pub tags: Option<NonEmptyVec<NonEmptyString>>,
}

impl From<FeedCreate> for feed::FeedCreate {
    fn from(value: FeedCreate) -> Self {
        Self {
            url: value.url,
            title: value.title,
            pinned: value.pinned,
            tags: value.tags,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedUpdate {
    #[schema(value_type = Option<String>, min_length = 1)]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub title: Option<Option<NonEmptyString>>,
    #[schema(nullable = false)]
    pub pinned: Option<bool>,
    #[schema(value_type = Option<Vec<String>>, nullable = false, min_length = 1)]
    pub tags: Option<NonEmptyVec<NonEmptyString>>,
}

impl From<FeedUpdate> for feed::FeedUpdate {
    fn from(value: FeedUpdate) -> Self {
        Self {
            title: value.title,
            pinned: value.pinned,
            tags: value.tags,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct FeedListQuery {
    #[param(nullable = false)]
    pub pinned: Option<bool>,
    #[param(nullable = false)]
    pub filter_by_tags: Option<bool>,
    #[param(min_length = 1, nullable = false)]
    #[serde(rename = "tag[]")]
    pub tags: Option<Vec<String>>,
}

impl From<FeedListQuery> for feed::FeedListQuery {
    fn from(value: FeedListQuery) -> Self {
        Self {
            pinned: value.pinned,
            tags: if value.filter_by_tags.unwrap_or(value.tags.is_some()) {
                value.tags
            } else {
                None
            },
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedDetect {
    #[schema(format = "uri")]
    pub url: Url,
}

impl From<FeedDetect> for feed::FeedDetect {
    fn from(value: FeedDetect) -> Self {
        Self { url: value.url }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedDetected {
    #[schema(format = "uri")]
    pub url: String,
    pub title: String,
}

impl From<feed::FeedDetected> for FeedDetected {
    fn from(value: feed::FeedDetected) -> Self {
        Self {
            url: value.url,
            title: value.title,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedProcessed {
    #[schema(format = "uri")]
    pub link: String,
    pub title: String,
}

impl From<feed::ProcessedFeed> for FeedProcessed {
    fn from(value: feed::ProcessedFeed) -> Self {
        Self {
            link: value.link.to_string(),
            title: value.title,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(untagged)]
pub enum DetectedResponse {
    Detected(Vec<FeedDetected>),
    Processed(FeedProcessed),
}

impl From<feed::DetectedResponse> for DetectedResponse {
    fn from(value: feed::DetectedResponse) -> Self {
        match value {
            feed::DetectedResponse::Detected(feeds) => {
                Self::Detected(feeds.into_iter().map(FeedDetected::from).collect())
            }
            feed::DetectedResponse::Processed(feed) => Self::Processed(feed.into()),
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    params(FeedListQuery),
    responses(ListResponse),
    operation_id = "listFeeds",
    description = "List user feeds",
    tag = FEEDS_TAG
)]
#[axum::debug_handler]
pub async fn list_feeds(
    State(service): State<Arc<FeedService>>,
    Query(query): Query<FeedListQuery>,
    session: Session,
) -> Result<ListResponse, Error> {
    match service.list_feeds(query.into(), session.user_id).await {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getFeed",
    description = "Get a feed by ID",
    tag = FEEDS_TAG
)]
#[axum::debug_handler]
pub async fn get_feed(
    State(service): State<Arc<FeedService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<GetResponse, Error> {
    match service.get_feed(id, session.user_id).await {
        Ok(data) => Ok(GetResponse::Ok(data.into())),
        Err(e) => match e {
            feed::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[utoipa::path(
    post,
    path = "",
    request_body = FeedCreate,
    responses(CreateResponse),
    operation_id = "createFeed",
    description = "Subscribe to a web feed",
    tag = FEEDS_TAG
  )]
#[axum::debug_handler]
pub async fn create_feed(
    State(service): State<Arc<FeedService>>,
    session: Session,
    Json(body): Json<FeedCreate>,
) -> Result<CreateResponse, Error> {
    match service.create_feed(body.into(), session.user_id).await {
        Ok(data) => Ok(CreateResponse::Created(data.into())),
        Err(e) => match e {
            feed::Error::Conflict(_) => Ok(CreateResponse::Conflict(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown(e.into())),
        },
    }
}

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = FeedUpdate,
    responses(UpdateResponse),
    operation_id = "updateFeed",
    description = "Update a feed by ID",
    tag = FEEDS_TAG
)]
#[axum::debug_handler]
pub async fn update_feed(
    State(service): State<Arc<FeedService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Json(body): Json<FeedUpdate>,
) -> Result<UpdateResponse, Error> {
    match service.update_feed(id, body.into(), session.user_id).await {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
        Err(e) => match e {
            feed::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
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
    operation_id = "deleteFeed",
    description = "Delete a feed by ID",
    tag = FEEDS_TAG
)]
#[axum::debug_handler]
pub async fn delete_feed(
    State(service): State<Arc<FeedService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<DeleteResponse, Error> {
    match service.delete_feed(id, session.user_id).await {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            feed::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[utoipa::path(
    post,
    path = "/detect",
    request_body = FeedDetect,
    responses(DetectResponse),
    operation_id = "detectFeeds",
    description = "Detects web feeds on a page",
    tag = FEEDS_TAG
  )]
#[axum::debug_handler]
pub async fn detect_feeds(
    State(service): State<Arc<FeedService>>,
    Json(body): Json<FeedDetect>,
) -> Result<DetectResponse, Error> {
    match service.detect_feeds(body.into()).await {
        Ok(data) => Ok(DetectResponse::Ok(data.into())),
        Err(feed::Error::Scraper(e)) => Ok(DetectResponse::BadGateway(BaseError {
            message: e.to_string(),
        })),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of feeds")]
    Ok(Paginated<Feed>),
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
    #[response(status = 200, description = "Feed by ID")]
    Ok(Feed),

    #[response(status = 404, description = "Feed not found")]
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
    #[response(status = 201, description = "Created feed")]
    Created(Feed),

    #[response(status = 409, description = "Feed not cached")]
    Conflict(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::Conflict(data) => (StatusCode::CONFLICT, Json(data)).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated feed")]
    Ok(Feed),

    #[response(status = 404, description = "Feed not found")]
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
    #[response(status = 204, description = "Successfully deleted feed")]
    NoContent,

    #[response(status = 404, description = "Feed not found")]
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

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum DetectResponse {
    #[response(status = 200, description = "Detected feeds")]
    Ok(DetectedResponse),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),

    #[response(status = 502, description = "Failed to fetch or parse feed")]
    BadGateway(BaseError),
}

impl IntoResponse for DetectResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
            Self::BadGateway(e) => (StatusCode::BAD_GATEWAY, e).into_response(),
        }
    }
}
