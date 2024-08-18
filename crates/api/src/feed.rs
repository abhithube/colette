use std::{any::Any, sync::Arc};

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_extra::extract::Query;
use axum_valid::Valid;
use colette_backup::{
    opml::{Opml, OpmlBody, OpmlOutline, OpmlOutlineType},
    BackupManager,
};
use colette_core::feed::{
    self, FeedCreateData, FeedFindManyFilters, FeedRepository, FeedScraper, FeedUpdateData,
};
use url::Url;
use uuid::Uuid;

use crate::{
    common::{BaseError, Error, FeedDetectedList, FeedList, Id, Paginated, Session},
    tag::{Tag, TagCreate},
};

#[derive(Clone, axum::extract::FromRef)]
pub struct FeedState {
    pub repository: Arc<dyn FeedRepository>,
    pub scraper: Arc<dyn FeedScraper>,
    pub opml: Arc<dyn BackupManager<T = Opml>>,
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
    components(schemas(
        Feed,
        FeedList,
        FeedDetectedList,
        FeedCreate,
        FeedUpdate,
        FeedDetect,
        FeedDetected,
        File
    ))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<FeedState> {
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
    #[schema(required)]
    pub title: Option<String>,
    pub original_title: String,
    #[schema(format = "uri", required)]
    pub url: Option<String>,
    #[schema(required)]
    pub folder_id: Option<Uuid>,
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
            original_title: value.original_title,
            url: value.url,
            folder_id: value.folder_id,
            tags: value.tags.map(|e| e.into_iter().map(Tag::from).collect()),
            unread_count: value.unread_count,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct FeedCreate {
    #[schema(format = "uri")]
    pub url: Url,
    pub folder_id: Option<Uuid>,
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct FeedUpdate {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    #[validate(length(min = 1))]
    pub title: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub folder_id: Option<Option<Uuid>>,
    #[schema(nullable = false)]
    pub tags: Option<Vec<TagCreate>>,
}

impl From<FeedUpdate> for FeedUpdateData {
    fn from(value: FeedUpdate) -> Self {
        Self {
            title: value.title,
            folder_id: value.folder_id,
            tags: value.tags.map(|e| e.into_iter().map(|e| e.title).collect()),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct ListFeedsQuery {
    #[param(nullable = false)]
    pub filter_by_tags: Option<bool>,
    #[param(min_length = 1, nullable = false)]
    #[serde(rename = "tag[]")]
    pub tags: Option<Vec<String>>,
}

impl From<ListFeedsQuery> for FeedFindManyFilters {
    fn from(value: ListFeedsQuery) -> Self {
        Self {
            tags: if value.filter_by_tags.unwrap_or(value.tags.is_some()) {
                value.tags
            } else {
                None
            },
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct FeedDetect {
    #[schema(format = "uri")]
    pub url: Url,
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedDetected {
    #[schema(format = "uri")]
    pub url: String,
    pub title: String,
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct File {
    #[allow(dead_code)]
    #[schema(format = "Binary")]
    pub data: String,
}

#[utoipa::path(
    get,
    path = "",
    params(ListFeedsQuery),
    responses(ListResponse),
    operation_id = "listFeeds",
    description = "List the active profile feeds"
)]
#[axum::debug_handler]
pub async fn list_feeds(
    State(repository): State<Arc<dyn FeedRepository>>,
    Query(query): Query<ListFeedsQuery>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .find_many_feeds(session.profile_id, None, None, Some(query.into()))
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
    description = "Get a feed by ID"
)]
#[axum::debug_handler]
pub async fn get_feed(
    State(repository): State<Arc<dyn FeedRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .find_one_feed(id, session.profile_id)
        .await
        .map(Feed::from);

    match result {
        Ok(data) => Ok(GetResponse::Ok(data)),
        Err(e) => match e {
            feed::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[utoipa::path(
  post,
  path = "",
  request_body = FeedCreate,
  responses(CreateResponse),
  operation_id = "createFeed",
  description = "Subscribe to a web feed"
)]
#[axum::debug_handler]
pub async fn create_feed(
    State(FeedState {
        repository,
        scraper,
        ..
    }): State<FeedState>,
    session: Session,
    Valid(Json(mut body)): Valid<Json<FeedCreate>>,
) -> Result<impl IntoResponse, Error> {
    let scraped = scraper.scrape(&mut body.url);
    if let Err(e) = scraped {
        return Ok(CreateResponse::BadGateway(BaseError {
            message: e.to_string(),
        }));
    }

    let result = repository
        .create_feed(FeedCreateData {
            url: body.url.into(),
            feed: scraped.unwrap(),
            folder_id: Some(body.folder_id),
            profile_id: session.profile_id,
        })
        .await
        .map(Feed::from);

    match result {
        Ok(data) => Ok(CreateResponse::Created(data)),
        _ => Err(Error::Unknown),
    }
}

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = FeedUpdate,
    responses(UpdateResponse),
    operation_id = "updateFeed",
    description = "Update a feed by ID"
)]
#[axum::debug_handler]
pub async fn update_feed(
    State(repository): State<Arc<dyn FeedRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Valid(Json(body)): Valid<Json<FeedUpdate>>,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .update_feed(id, session.profile_id, body.into())
        .await
        .map(Feed::from);

    match result {
        Ok(data) => Ok(UpdateResponse::Ok(data)),
        Err(e) => match e {
            feed::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
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
    description = "Delete a feed by ID"
)]
#[axum::debug_handler]
pub async fn delete_feed(
    State(repository): State<Arc<dyn FeedRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository.delete_feed(id, session.profile_id).await;

    match result {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            feed::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[utoipa::path(
    post,
    path = "/detect",
    request_body = FeedDetect,
    responses(DetectResponse),
    operation_id = "detectFeeds",
    description = "Detects web feeds on a page"
  )]
#[axum::debug_handler]
pub async fn detect_feeds(
    State(scraper): State<Arc<dyn FeedScraper>>,
    Valid(Json(mut body)): Valid<Json<FeedDetect>>,
) -> Result<impl IntoResponse, Error> {
    let urls = scraper.detect(&mut body.url);
    if let Err(e) = urls {
        return Ok(DetectResponse::BadGateway(BaseError {
            message: e.to_string(),
        }));
    }

    let mut feeds: Vec<FeedDetected> = vec![];

    for mut url in urls.unwrap().into_iter() {
        let feed = scraper.scrape(&mut url);
        if let Err(e) = feed {
            return Ok(DetectResponse::BadGateway(BaseError {
                message: e.to_string(),
            }));
        }

        feeds.push(FeedDetected {
            url: url.into(),
            title: feed.unwrap().title,
        })
    }

    Ok(DetectResponse::Ok(Paginated::<FeedDetected> {
        data: feeds,
        cursor: None,
    }))
}

#[utoipa::path(
    post,
    path = "/import",
    request_body(content = File, content_type = "multipart/form-data"),
    responses(ImportResponse),
    operation_id = "importFeeds",
    description = "Import OPML feeds into profile"
)]
#[axum::debug_handler]
pub async fn import_feeds(
    State(state): State<FeedState>,
    session: Session,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, Error> {
    let Ok(Some(field)) = multipart.next_field().await else {
        return Err(Error::Unknown);
    };

    let raw = field.text().await.map_err(|_| Error::Unknown)?;

    for outline in state
        .opml
        .import(&raw)
        .map_err(|_| Error::Unknown)?
        .body
        .outlines
    {
        if let Some(xml_url) = outline.xml_url {
            create_feed(
                State(state.clone()),
                session.clone(),
                Valid(Json(FeedCreate {
                    url: xml_url,
                    folder_id: None,
                })),
            )
            .await?;
        }
    }

    Ok(ImportResponse::NoContent)
}

#[utoipa::path(
    post,
    path = "/export",
    responses(ExportResponse),
    operation_id = "exportFeeds",
    description = "Export OPML feeds from profile"
)]
#[axum::debug_handler]
pub async fn export_feeds(
    State(state): State<FeedState>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let response = list_feeds(
        State(state.repository),
        Query(ListFeedsQuery {
            filter_by_tags: None,
            tags: None,
        }),
        session,
    )
    .await?;

    let ListResponse::Ok(feeds) = (&response as &dyn Any)
        .downcast_ref::<ListResponse>()
        .unwrap();

    let data = feeds
        .data
        .iter()
        .cloned()
        .map(|e| OpmlOutline {
            outline_type: Some(OpmlOutlineType::default()),
            text: e.title.clone().unwrap_or(e.original_title.clone()),
            title: Some(e.title.unwrap_or(e.original_title)),
            xml_url: e.url.and_then(|e| Url::parse(&e).ok()),
            html_url: Url::parse(&e.link).ok(),
            children: None,
        })
        .collect::<Vec<_>>();

    let opml = Opml {
        body: OpmlBody { outlines: data },
        ..Default::default()
    };

    let data = state.opml.export(opml).map_err(|_| Error::Unknown)?;

    Ok(ExportResponse::Ok(data.as_bytes().into()))
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
