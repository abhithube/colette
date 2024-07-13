use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_valid::Valid;
use chrono::{DateTime, Utc};
use colette_core::feeds::{self, FeedsService};

use crate::{
    common::{self, BaseError, Context, FeedList, Id, Paginated, ValidationError},
    error::Error,
    session::Session,
};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(list_feeds, get_feed, create_feed, delete_feed),
    components(schemas(Feed, CreateFeed))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<Context> {
        Router::new().nest(
            "/feeds",
            Router::new()
                .route("/", routing::get(list_feeds).post(create_feed))
                .route("/:id", routing::get(get_feed).delete(delete_feed)),
        )
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
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = service
        .list(session.into())
        .await
        .map(Paginated::<Feed>::from);

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
    let result = service.get(id, session.into()).await.map(Feed::from);

    match result {
        Ok(data) => Ok(GetResponse::Ok(data)),
        Err(e) => match e {
            feeds::Error::NotFound(_) => Ok(GetResponse::NotFound(common::BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[utoipa::path(
  post,
  path = "",
  request_body = CreateFeed,
  responses(CreateResponse),
  operation_id = "createFeed",
  description = "Subscribe to a web feed",
  tag = "Feeds"
)]
#[axum::debug_handler]
pub async fn create_feed(
    State(service): State<Arc<FeedsService>>,
    session: Session,
    Valid(Json(body)): Valid<Json<CreateFeed>>,
) -> Result<impl IntoResponse, Error> {
    let result = service
        .create(body.into(), session.into())
        .await
        .map(Feed::from);

    match result {
        Ok(data) => Ok(CreateResponse::Created(data)),
        Err(e) => match e {
            feeds::Error::Scraper(_) => Ok(CreateResponse::BadGateway(common::BaseError {
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
    let result = service.delete(id, session.into()).await;

    match result {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            feeds::Error::NotFound(_) => Ok(DeleteResponse::NotFound(common::BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub id: String,
    #[schema(format = "uri")]
    pub link: String,
    pub title: String,
    #[schema(format = "uri")]
    pub url: Option<String>,
    pub custom_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub unread_count: Option<i64>,
}

impl From<colette_core::Feed> for Feed {
    fn from(value: colette_core::Feed) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            url: value.url,
            custom_title: value.custom_title,
            created_at: value.created_at,
            updated_at: value.updated_at,
            unread_count: value.unread_count,
        }
    }
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeed {
    #[validate(url(message = "not a valid URL"))]
    pub url: String,
}

impl From<CreateFeed> for feeds::CreateFeed {
    fn from(value: CreateFeed) -> Self {
        Self { url: value.url }
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

#[derive(Debug, serde::Serialize, utoipa::ToResponse)]
#[serde(rename_all = "camelCase")]
#[response(description = "Invalid input")]
pub struct CreateValidationErrors {
    url: Option<Vec<ValidationError>>,
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created feed")]
    Created(Feed),

    #[allow(dead_code)]
    #[response(status = 422)]
    UnprocessableEntity(#[to_response] CreateValidationErrors),

    #[response(status = 502, description = "Failed to fetch or parse feed")]
    BadGateway(BaseError),
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::UnprocessableEntity(e) => {
                (StatusCode::UNPROCESSABLE_ENTITY, Json(e)).into_response()
            }
            Self::BadGateway(e) => (StatusCode::BAD_GATEWAY, e).into_response(),
        }
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
