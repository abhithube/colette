use axum::{
    Json,
    extract::{
        FromRequestParts,
        rejection::{JsonRejection, QueryRejection},
    },
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use colette_core::{auth, common};
use tower_sessions::session;
use uuid::Uuid;

pub const AUTH_TAG: &str = "Auth";
pub const BACKUPS_TAG: &str = "Backups";
// pub const COLLECTIONS_TAG: &str = "Collections";
pub const BOOKMARKS_TAG: &str = "Bookmarks";
pub const FEED_ENTRIES_TAG: &str = "Feed Entries";
pub const FEEDS_TAG: &str = "Feeds";
pub const FOLDERS_TAG: &str = "Folders";
pub const LIBRARY_TAG: &str = "Library";
// pub const SMART_FEEDS_TAG: &str = "Smart Feeds";
pub const TAGS_TAG: &str = "Tags";

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[into_params(names("id"))]
pub struct Id(pub Uuid);

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Paginated<T: utoipa::ToSchema> {
    pub data: Vec<T>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl<T, U> From<common::Paginated<U>> for Paginated<T>
where
    T: From<U> + utoipa::ToSchema,
{
    fn from(value: common::Paginated<U>) -> Self {
        Self {
            data: value.data.into_iter().map(T::from).collect(),
            cursor: value.cursor,
        }
    }
}

pub const SESSION_KEY: &str = "session";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub user_id: Uuid,
}

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
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

    #[error(transparent)]
    Unknown(#[from] CoreError),
}

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error(transparent)]
    Auth(#[from] colette_core::auth::Error),

    #[error(transparent)]
    Backup(#[from] colette_core::backup::Error),

    #[error(transparent)]
    Bookmark(#[from] colette_core::bookmark::Error),

    #[error(transparent)]
    Feed(#[from] colette_core::feed::Error),

    #[error(transparent)]
    FeedEntry(#[from] colette_core::feed_entry::Error),

    #[error(transparent)]
    Folder(#[from] colette_core::folder::Error),

    #[error(transparent)]
    Library(#[from] colette_core::library::Error),

    #[error(transparent)]
    Tag(#[from] colette_core::tag::Error),

    #[error(transparent)]
    User(#[from] colette_core::user::Error),
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
            _ => {
                tracing::error!("{:?}", self);

                (StatusCode::INTERNAL_SERVER_ERROR, e).into_response()
            }
        }
    }
}
