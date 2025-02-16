use std::sync::Arc;

use axum::{
    Json,
    extract::{
        FromRef, FromRequestParts, OptionalFromRequestParts,
        rejection::{JsonRejection, QueryRejection},
    },
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use colette_core::{api_key::ApiKeyService, auth, common};
use tower_sessions::session;
use uuid::Uuid;

pub const SESSION_KEY: &str = "session";

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[into_params(names("id"))]
pub struct Id(pub Uuid);

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct NonEmptyString(String);

impl TryFrom<String> for NonEmptyString {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ValidationError::Empty);
        }

        Ok(NonEmptyString(value))
    }
}

impl From<NonEmptyString> for String {
    fn from(value: NonEmptyString) -> Self {
        value.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("cannot be empty")]
    Empty,
}

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

#[derive(Debug, Clone)]
pub struct ApiKey(String);

impl<S> OptionalFromRequestParts<S> for ApiKey
where
    Arc<ApiKeyService>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let Some(header) = parts.headers.get("X-Api-Key") else {
            return Ok(None);
        };

        let header_raw = header
            .to_str()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Some(ApiKey(header_raw.into())))
    }
}

#[derive(Debug, Clone)]
pub struct AuthUser(pub Uuid);

impl<S> FromRequestParts<S> for AuthUser
where
    Arc<ApiKeyService>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session_store = tower_sessions::Session::from_request_parts(parts, state)
            .await
            .unwrap();

        let user_id = session_store
            .get::<Uuid>(SESSION_KEY)
            .await
            .map_err(|_| Error::Auth(auth::Error::NotAuthenticated))?;

        if let Some(user_id) = user_id {
            return Ok(AuthUser(user_id));
        }

        let api_key = ApiKey::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::Auth(auth::Error::NotAuthenticated))?
            .ok_or_else(|| Error::Auth(auth::Error::NotAuthenticated))?;

        let service = Arc::<ApiKeyService>::from_ref(state);

        let user_id = service
            .validate_api_key(api_key.0)
            .await
            .map_err(|_| Error::Auth(auth::Error::NotAuthenticated))?;

        Ok(AuthUser(user_id))
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
    ApiKey(#[from] colette_core::api_key::Error),

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
