use axum::{
    async_trait,
    extract::{
        rejection::{JsonRejection, QueryRejection},
        FromRequestParts,
    },
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use colette_core::{auth, common};
use serde::{Deserialize, Serialize};
use tower_sessions::session;
use uuid::Uuid;

use crate::{
    bookmarks::Bookmark,
    entries::Entry,
    feeds::{Feed, FeedDetected},
    profiles::Profile,
    tags::Tag,
};

#[derive(Clone, Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(names("id"))]
pub struct Id(pub Uuid);

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[aliases(BookmarkList = Paginated<Bookmark>, FeedDetectedList = Paginated<FeedDetected>, FeedList = Paginated<Feed>, ProfileList = Paginated<Profile>, TagList = Paginated<Tag>)]
#[serde(rename_all = "camelCase")]
pub struct Paginated<T: serde::Serialize> {
    pub has_more: bool,
    pub data: Vec<T>,
}

impl<T, U> From<common::Paginated<U>> for Paginated<T>
where
    T: From<U> + serde::Serialize,
{
    fn from(value: common::Paginated<U>) -> Self {
        Self {
            has_more: value.has_more,
            data: value.data.into_iter().map(T::from).collect(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[aliases(EntryList = CursorPaginated<Entry>)]
#[serde(rename_all = "camelCase")]
pub struct CursorPaginated<T: serde::Serialize> {
    pub data: Vec<T>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl<T, U> From<common::CursorPaginated<U>> for CursorPaginated<T>
where
    T: From<U> + serde::Serialize,
{
    fn from(value: common::CursorPaginated<U>) -> Self {
        Self {
            data: value.data.into_iter().map(T::from).collect(),
            cursor: value.cursor,
        }
    }
}

pub const SESSION_KEY: &str = "session";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub user_id: Uuid,
    pub profile_id: Uuid,
}

impl From<Session> for common::Session {
    fn from(value: Session) -> Self {
        Self {
            user_id: value.user_id,
            profile_id: value.profile_id,
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Session
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session_store = tower_sessions::Session::from_request_parts(req, state)
            .await
            .map_err(|_| Error::Auth(auth::Error::NotAuthenticated))?;

        let session = session_store
            .get::<Session>(SESSION_KEY)
            .await
            .map_err(|_| Error::Auth(auth::Error::NotAuthenticated))?
            .ok_or(Error::Auth(auth::Error::NotAuthenticated))?;

        Ok(session)
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[schema(title = "Error")]
pub struct BaseError {
    pub message: String,
}

impl IntoResponse for BaseError {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    QueryRejection(#[from] QueryRejection),

    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),

    #[error(transparent)]
    Session(#[from] session::Error),

    #[error(transparent)]
    Auth(#[from] auth::Error),

    #[error("Unknown error")]
    Unknown,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let e = BaseError {
            message: self.to_string(),
        };

        match self {
            Error::QueryRejection(_) => (StatusCode::BAD_REQUEST, e).into_response(),
            Error::JsonRejection(_) => (StatusCode::BAD_REQUEST, e).into_response(),
            Error::Auth(auth::Error::NotAuthenticated) => {
                (StatusCode::UNAUTHORIZED, e).into_response()
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
    }
}
