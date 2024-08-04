use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_extra::extract::Query;
use axum_valid::Valid;
use colette_core::feeds::{
    self, CreateFeed, DetectedFeed, FeedsService, ImportFeeds, ListFeedsParams, UpdateFeed,
};
use url::Url;
use uuid::Uuid;

use crate::{
    common::{BaseError, Error, FeedDetectedList, FeedList, Id, Paginated, Session},
    tags::Tag,
};

#[derive(Clone, axum::extract::FromRef)]
pub struct FeedsState {
    pub service: Arc<FeedsService>,
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        list_feeds,
        get_feed,
        create_feed,
        update_feed,
        delete_feed,
        detect_feeds,
        import_feeds,
        export_feeds
    ),
    components(schemas(Feed, FeedCreate, FeedUpdate, FeedDetect, FeedDetected, File))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<FeedsState> {
        Router::new().nest(
            "/feeds",
            Router::new()
                .route("/", routing::get(list_feeds).post(create_feed))
                .route(
                    "/:id",
                    routing::get(get_feed)
                        .patch(update_feed)
                        .delete(delete_feed),
                )
                .route("/detect", routing::post(detect_feeds))
                .route("/import", routing::post(import_feeds))
                .route("/export", routing::post(export_feeds)),
        )
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub id: Uuid,
    #[schema(format = "uri")]
    pub link: String,
    pub title: String,
    #[schema(format = "uri", required)]
    pub url: Option<String>,
    #[schema(required = false)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
    #[schema(nullable = false)]
    pub unread_count: Option<i64>,
}

impl From<colette_core::Feed> for Feed {
    fn from(value: colette_core::Feed) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            url: value.url,
            tags: value.tags.into_iter().map(Tag::from).collect(),
            unread_count: value.unread_count,
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    responses(ListResponse),
    operation_id = "listFeeds",
    description = "List the active profile feeds",
    tag = "Feeds"
)]
#[axum::debug_handler]
pub async fn list_feeds(
    State(service): State<Arc<FeedsService>>,
    Query(query): Query<ListFeedsQuery>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service
        .list(query.into(), session.into())
        .await
        .map(Paginated::<Feed>::from)
    {
        Ok(data) => Ok(ListResponse::Ok(data)),
        _ => Err(Error::Unknown),
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct ListFeedsQuery {
    #[param(nullable = false)]
    pub tags: Option<Vec<Uuid>>,
}

impl From<ListFeedsQuery> for ListFeedsParams {
    fn from(value: ListFeedsQuery) -> Self {
        Self { tags: value.tags }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of profiles")]
    Ok(FeedList),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getFeed",
    description = "Get a feed by ID",
    tag = "Feeds"
)]
#[axum::debug_handler]
pub async fn get_feed(
    State(service): State<Arc<FeedsService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service.get(id, session.into()).await.map(Feed::from) {
        Ok(data) => Ok(GetResponse::Ok(data)),
        Err(e) => match e {
            feeds::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Debug, utoipa::IntoResponses)]
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

#[utoipa::path(
  post,
  path = "",
  request_body = FeedCreate,
  responses(CreateResponse),
  operation_id = "createFeed",
  description = "Subscribe to a web feed",
  tag = "Feeds"
)]
#[axum::debug_handler]
pub async fn create_feed(
    State(service): State<Arc<FeedsService>>,
    session: Session,
    Valid(Json(body)): Valid<Json<FeedCreate>>,
) -> Result<impl IntoResponse, Error> {
    match service
        .create(body.into(), session.into())
        .await
        .map(Feed::from)
    {
        Ok(data) => Ok(CreateResponse::Created(data)),
        Err(e) => match e {
            feeds::Error::Scraper(_) => Ok(CreateResponse::BadGateway(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct FeedCreate {
    #[schema(format = "uri")]
    pub url: Url,
}

impl From<FeedCreate> for CreateFeed {
    fn from(value: FeedCreate) -> Self {
        Self { url: value.url }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created feed")]
    Created(Feed),

    #[allow(dead_code)]
    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),

    #[response(status = 502, description = "Failed to fetch or parse feed")]
    BadGateway(BaseError),
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
            Self::BadGateway(e) => (StatusCode::BAD_GATEWAY, e).into_response(),
        }
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
    tag = "Feeds"
)]
#[axum::debug_handler]
pub async fn update_feed(
    State(service): State<Arc<FeedsService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Valid(Json(body)): Valid<Json<FeedUpdate>>,
) -> Result<impl IntoResponse, Error> {
    match service
        .update(id, body.into(), session.into())
        .await
        .map(Feed::from)
    {
        Ok(data) => Ok(UpdateResponse::Ok(data)),
        Err(e) => match e {
            feeds::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct FeedUpdate {
    pub tags: Option<Vec<Uuid>>,
}

impl From<FeedUpdate> for UpdateFeed {
    fn from(value: FeedUpdate) -> Self {
        Self { tags: value.tags }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated feed")]
    Ok(Feed),

    #[response(status = 404, description = "Feed not found")]
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

#[utoipa::path(
    delete,
    path = "/{id}",
    params(Id),
    responses(DeleteResponse),
    operation_id = "deleteFeed",
    description = "Delete a feed by ID",
    tag = "Feeds"
)]
#[axum::debug_handler]
pub async fn delete_feed(
    State(service): State<Arc<FeedsService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service.delete(id, session.into()).await {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            feeds::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Debug, utoipa::IntoResponses)]
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

#[utoipa::path(
    post,
    path = "/detect",
    request_body = FeedDetect,
    responses(DetectResponse),
    operation_id = "detectFeeds",
    description = "Detects web feeds on a page",
    tag = "Feeds"
  )]
#[axum::debug_handler]
pub async fn detect_feeds(
    State(service): State<Arc<FeedsService>>,
    Valid(Json(body)): Valid<Json<FeedDetect>>,
) -> Result<impl IntoResponse, Error> {
    match service
        .detect(body.into())
        .await
        .map(Paginated::<FeedDetected>::from)
    {
        Ok(data) => Ok(DetectResponse::Ok(data)),
        Err(e) => match e {
            feeds::Error::Scraper(_) => Ok(DetectResponse::BadGateway(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct FeedDetect {
    #[schema(format = "uri")]
    pub url: Url,
}

impl From<FeedDetect> for feeds::DetectFeeds {
    fn from(value: FeedDetect) -> Self {
        Self { url: value.url }
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedDetected {
    #[schema(format = "uri")]
    pub url: String,
    pub title: String,
}

impl From<DetectedFeed> for FeedDetected {
    fn from(value: DetectedFeed) -> Self {
        Self {
            url: value.url,
            title: value.title,
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum DetectResponse {
    #[response(status = 201, description = "Detected feeds")]
    Ok(FeedDetectedList),

    #[allow(dead_code)]
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

#[utoipa::path(
    post,
    path = "/import",
    request_body(content = File, content_type = "multipart/form-data"),
    responses(ImportResponse),
    operation_id = "importFeeds",
    description = "Import OPML feeds into profile",
    tag = "Feeds"
)]
#[axum::debug_handler]
pub async fn import_feeds(
    State(service): State<Arc<FeedsService>>,
    session: Session,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, Error> {
    let Ok(Some(field)) = multipart.next_field().await else {
        return Err(Error::Unknown);
    };

    let raw = field.text().await.map_err(|_| Error::Unknown)?;

    match service.import(ImportFeeds { raw }, session.into()).await {
        Ok(()) => Ok(ImportResponse::NoContent),
        _ => Err(Error::Unknown),
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct File {
    #[allow(dead_code)]
    #[schema(format = "Binary")]
    pub data: String,
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ImportResponse {
    #[response(status = 204, description = "Successfully started import")]
    NoContent,
}

impl IntoResponse for ImportResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
        }
    }
}

#[utoipa::path(
    post,
    path = "/export",
    responses(ExportResponse),
    operation_id = "exportFeeds",
    description = "Export OPML feeds from profile",
    tag = "Feeds"
)]
#[axum::debug_handler]
pub async fn export_feeds(
    State(service): State<Arc<FeedsService>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service.export(session.into()).await {
        Ok(data) => Ok(ExportResponse::Ok(data.as_bytes().into())),
        _ => Err(Error::Unknown),
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ExportResponse {
    #[response(
        status = 200,
        description = "OPML file",
        content_type = "application/octet-stream"
    )]
    Ok(Box<[u8]>),
}

impl IntoResponse for ExportResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => data.into_response(),
        }
    }
}
