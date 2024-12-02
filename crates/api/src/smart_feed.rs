use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use colette_core::{
    common::{NonEmptyString, NonEmptyVec},
    smart_feed::{self, SmartFeedService},
};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    common::{BaseError, Error, Id, Session, SMART_FEEDS_TAG},
    Paginated,
};

#[derive(Clone, axum::extract::FromRef)]
pub struct SmartFeedState {
    service: SmartFeedService,
}

impl SmartFeedState {
    pub fn new(service: SmartFeedService) -> Self {
        Self { service }
    }
}

#[derive(OpenApi)]
#[openapi(components(schemas(
    SmartFeed,
    Paginated<SmartFeed>,
    SmartFeedCreate,
    SmartFeedUpdate,
    SmartFeedFilter,
    TextOperation,
    BooleanOperation,
    DateOperation
)))]
pub struct SmartFeedApi;

impl SmartFeedApi {
    pub fn router() -> OpenApiRouter<SmartFeedState> {
        OpenApiRouter::with_openapi(SmartFeedApi::openapi())
            .routes(routes!(list_smart_feeds, create_smart_feed))
            .routes(routes!(
                get_smart_feed,
                update_smart_feed,
                delete_smart_feed
            ))
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SmartFeed {
    pub id: Uuid,
    pub title: String,
    #[schema(nullable = false)]
    pub unread_count: Option<i64>,
}

impl From<colette_core::SmartFeed> for SmartFeed {
    fn from(value: colette_core::SmartFeed) -> Self {
        Self {
            id: value.id,
            title: value.title,
            unread_count: value.unread_count,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SmartFeedCreate {
    #[schema(value_type = String, min_length = 1)]
    pub title: NonEmptyString,
    #[schema(value_type = Option<Vec<SmartFeedFilter>>, nullable = false, min_length = 1)]
    pub filters: Option<NonEmptyVec<SmartFeedFilter>>,
}

impl From<SmartFeedCreate> for smart_feed::SmartFeedCreate {
    fn from(value: SmartFeedCreate) -> Self {
        Self {
            title: value.title,
            filters: value
                .filters
                .map(|e| e.into_iter().map(|e| e.into()).collect()),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SmartFeedUpdate {
    #[schema(value_type = Option<String>, min_length = 1)]
    pub title: Option<NonEmptyString>,
    #[schema(value_type = Option<Vec<SmartFeedFilter>>, nullable = false, min_length = 1)]
    pub filters: Option<NonEmptyVec<SmartFeedFilter>>,
}

impl From<SmartFeedUpdate> for smart_feed::SmartFeedUpdate {
    fn from(value: SmartFeedUpdate) -> Self {
        Self {
            title: value.title,
            filters: value
                .filters
                .map(|e| e.into_iter().map(|e| e.into()).collect()),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase", tag = "field", content = "operation")]
pub enum SmartFeedFilter {
    Link(TextOperation),
    Title(TextOperation),
    PublishedAt(DateOperation),
    Description(TextOperation),
    Author(TextOperation),
    HasRead(BooleanOperation),
}

impl From<SmartFeedFilter> for smart_feed::SmartFeedFilter {
    fn from(value: SmartFeedFilter) -> Self {
        match value {
            SmartFeedFilter::Link(op) => Self::Link(op.into()),
            SmartFeedFilter::Title(op) => Self::Title(op.into()),
            SmartFeedFilter::PublishedAt(op) => Self::PublishedAt(op.into()),
            SmartFeedFilter::Description(op) => Self::Description(op.into()),
            SmartFeedFilter::Author(op) => Self::Author(op.into()),
            SmartFeedFilter::HasRead(op) => Self::HasRead(op.into()),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum TextOperation {
    Equals(String),
    DoesNotEqual(String),
    Contains(String),
    DoesNotContain(String),
}

impl From<TextOperation> for smart_feed::TextOperation {
    fn from(value: TextOperation) -> Self {
        match value {
            TextOperation::Equals(data) => Self::Equals(data),
            TextOperation::DoesNotEqual(data) => Self::DoesNotEqual(data),
            TextOperation::Contains(data) => Self::Contains(data),
            TextOperation::DoesNotContain(data) => Self::DoesNotContain(data),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BooleanOperation {
    pub value: bool,
}

impl From<BooleanOperation> for smart_feed::BooleanOperation {
    fn from(value: BooleanOperation) -> Self {
        Self { value: value.value }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum DateOperation {
    Equals(DateTime<Utc>),
    GreaterThan(DateTime<Utc>),
    LessThan(DateTime<Utc>),
    InLast(i64),
}

impl From<DateOperation> for smart_feed::DateOperation {
    fn from(value: DateOperation) -> Self {
        match value {
            DateOperation::Equals(data) => Self::Equals(data),
            DateOperation::GreaterThan(data) => Self::GreaterThan(data),
            DateOperation::LessThan(data) => Self::LessThan(data),
            DateOperation::InLast(data) => Self::InLast(data),
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    responses(ListResponse),
    operation_id = "listSmartFeeds",
    description = "List the active profile smart feeds",
    tag = SMART_FEEDS_TAG
)]
#[axum::debug_handler]
pub async fn list_smart_feeds(
    State(service): State<SmartFeedService>,
    session: Session,
) -> Result<ListResponse, Error> {
    match service.list_smart_feeds(session.profile_id).await {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getSmartFeed",
    description = "Get a smart feed by ID",
    tag = SMART_FEEDS_TAG
)]
#[axum::debug_handler]
pub async fn get_smart_feed(
    State(service): State<SmartFeedService>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<GetResponse, Error> {
    match service.get_smart_feed(id, session.profile_id).await {
        Ok(data) => Ok(GetResponse::Ok(data.into())),
        Err(e) => match e {
            smart_feed::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[utoipa::path(
    post,
    path = "",
    request_body = SmartFeedCreate,
    responses(CreateResponse),
    operation_id = "createSmartFeed",
    description = "Create an auto-updating feed base on entry filters",
    tag = SMART_FEEDS_TAG
  )]
#[axum::debug_handler]
pub async fn create_smart_feed(
    State(service): State<SmartFeedService>,
    session: Session,
    Json(body): Json<SmartFeedCreate>,
) -> Result<CreateResponse, Error> {
    match service
        .create_smart_feed(body.into(), session.profile_id)
        .await
    {
        Ok(data) => Ok(CreateResponse::Created(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = SmartFeedUpdate,
    responses(UpdateResponse),
    operation_id = "updateSmartFeed",
    description = "Update a smart feed by ID",
    tag = SMART_FEEDS_TAG
)]
#[axum::debug_handler]
pub async fn update_smart_feed(
    State(service): State<SmartFeedService>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Json(body): Json<SmartFeedUpdate>,
) -> Result<UpdateResponse, Error> {
    match service
        .update_smart_feed(id, body.into(), session.profile_id)
        .await
    {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
        Err(e) => match e {
            smart_feed::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
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
    operation_id = "deleteSmartFeed",
    description = "Delete a smart feed by ID",
    tag = SMART_FEEDS_TAG
)]
#[axum::debug_handler]
pub async fn delete_smart_feed(
    State(service): State<SmartFeedService>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<DeleteResponse, Error> {
    match service.delete_smart_feed(id, session.profile_id).await {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            smart_feed::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of smart feeds")]
    Ok(Paginated<SmartFeed>),
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
    #[response(status = 200, description = "Smart feed by ID")]
    Ok(SmartFeed),

    #[response(status = 404, description = "Smart feed not found")]
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
    #[response(status = 201, description = "Created smart feed")]
    Created(SmartFeed),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated smart feed")]
    Ok(SmartFeed),

    #[response(status = 404, description = "Smart feed not found")]
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
    #[response(status = 204, description = "Successfully deleted smart feed")]
    NoContent,

    #[response(status = 404, description = "Smart feed not found")]
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
