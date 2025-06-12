use std::{ops::Range, sync::Arc};

use axum::{
    extract::{FromRequestParts, Request, State},
    http::{StatusCode, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::headers::{Authorization, HeaderMapExt, authorization::Bearer};
use chrono::{DateTime, Utc};
use colette_core::{
    User, api_key::ApiKeyService, auth::AuthService, bookmark::BookmarkService,
    collection::CollectionService, common, feed::FeedService, feed_entry::FeedEntryService, filter,
    job::JobService, stream::StreamService, subscription::SubscriptionService,
    subscription_entry::SubscriptionEntryService, tag::TagService,
};
use url::Url;
use uuid::Uuid;

/// API config
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// OIDC config
    pub oidc: Option<OidcConfig>,
    /// Storage config
    pub storage: StorageConfig,
}

/// API OIDC config
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OidcConfig {
    /// OIDC client ID
    pub client_id: String,
    /// OIDC issuer URL
    #[schema(value_type = Url)]
    pub issuer: String,
    /// OIDC redirect URI
    #[schema(value_type = Url)]
    pub redirect_uri: String,
}

/// API storage config
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StorageConfig {
    /// Base URL for the image storage server
    pub base_url: Url,
}

#[derive(Clone, axum::extract::FromRef)]
pub struct ApiState {
    pub api_key_service: Arc<ApiKeyService>,
    pub auth_service: Arc<AuthService>,
    pub bookmark_service: Arc<BookmarkService>,
    pub collection_service: Arc<CollectionService>,
    pub feed_service: Arc<FeedService>,
    pub feed_entry_service: Arc<FeedEntryService>,
    pub job_service: Arc<JobService>,
    pub stream_service: Arc<StreamService>,
    pub subscription_service: Arc<SubscriptionService>,
    pub subscription_entry_service: Arc<SubscriptionEntryService>,
    pub tag_service: Arc<TagService>,
    pub config: Config,
}

#[derive(axum::extract::FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub(crate) struct Path<T>(pub(crate) T);

#[derive(axum::extract::FromRequestParts)]
#[from_request(via(axum_extra::extract::Query), rejection(ApiError))]
pub(crate) struct Query<T>(pub(crate) T);

#[derive(axum::extract::FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub(crate) struct Json<T>(pub(crate) T);

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[into_params(names("id"))]
pub(crate) struct Id(
    /// Unique identifier of the resource
    pub(crate) Uuid,
);

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct NonEmptyString(String);

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
pub(crate) enum ValidationError {
    #[error("cannot be empty")]
    Empty,
}

/// Paginated list of results
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Paginated<T: utoipa::ToSchema> {
    /// Current set of results
    pub(crate) data: Vec<T>,
    /// Pagination cursor, only present if more results are available
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cursor: Option<String>,
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

pub(crate) async fn add_user_extension(
    State(state): State<ApiState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    req.extensions_mut().insert(None::<User>);

    if let Some(Authorization(bearer)) = req.headers().typed_get::<Authorization<Bearer>>() {
        let claims = state
            .auth_service
            .validate_access_token(bearer.token())
            .await
            .map_err(|_| ApiError::not_authenticated())?;

        let auth = Auth {
            user_id: claims.sub,
        };

        req.extensions_mut().insert(auth.clone());
        req.extensions_mut().insert(Some(auth));
    } else if let Some(header) = req.headers().get("X-Api-Key").and_then(|e| e.to_str().ok()) {
        let Ok(api_key) = state.api_key_service.validate_api_key(header.into()).await else {
            tracing::debug!("invalid API key");

            return Err(ApiError::not_authenticated());
        };

        let Ok(user) = state.auth_service.get_user(api_key.user_id).await else {
            tracing::debug!("user not found");

            return Err(ApiError::not_authenticated());
        };

        let auth = Auth { user_id: user.id };

        req.extensions_mut().insert(auth.clone());
        req.extensions_mut().insert(Some(auth));
    }

    Ok(next.run(req).await)
}

#[derive(Debug, Clone)]
pub(crate) struct Auth {
    pub(crate) user_id: Uuid,
}

impl<S: Send + Sync> FromRequestParts<S> for Auth {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<Auth>().cloned().ok_or_else(|| {
            tracing::debug!("failed to extract authenticated user");

            ApiError::not_authenticated()
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum TextOp {
    Equals(String),
    Contains(String),
    StartsWith(String),
    EndsWith(String),
}

impl From<TextOp> for filter::TextOp {
    fn from(value: TextOp) -> Self {
        match value {
            TextOp::Equals(value) => Self::Equals(value),
            TextOp::Contains(value) => Self::Contains(value),
            TextOp::StartsWith(value) => Self::StartsWith(value),
            TextOp::EndsWith(value) => Self::EndsWith(value),
        }
    }
}

impl From<filter::TextOp> for TextOp {
    fn from(value: filter::TextOp) -> Self {
        match value {
            filter::TextOp::Equals(value) => Self::Equals(value),
            filter::TextOp::Contains(value) => Self::Contains(value),
            filter::TextOp::StartsWith(value) => Self::StartsWith(value),
            filter::TextOp::EndsWith(value) => Self::EndsWith(value),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum NumberOp {
    Equals(f64),
    GreaterThan(f64),
    LessThan(f64),
    Between { start: f64, end: f64 },
}

impl From<NumberOp> for filter::NumberOp {
    fn from(value: NumberOp) -> Self {
        match value {
            NumberOp::Equals(value) => Self::Equals(value),
            NumberOp::GreaterThan(value) => Self::GreaterThan(value),
            NumberOp::LessThan(value) => Self::LessThan(value),
            NumberOp::Between { start, end } => Self::Between(Range { start, end }),
        }
    }
}

impl From<filter::NumberOp> for NumberOp {
    fn from(value: filter::NumberOp) -> Self {
        match value {
            filter::NumberOp::Equals(value) => Self::Equals(value),
            filter::NumberOp::GreaterThan(value) => Self::GreaterThan(value),
            filter::NumberOp::LessThan(value) => Self::LessThan(value),
            filter::NumberOp::Between(value) => Self::Between {
                start: value.start,
                end: value.end,
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum BooleanOp {
    Equals(bool),
}

impl From<BooleanOp> for filter::BooleanOp {
    fn from(value: BooleanOp) -> Self {
        match value {
            BooleanOp::Equals(value) => Self::Equals(value),
        }
    }
}

impl From<filter::BooleanOp> for BooleanOp {
    fn from(value: filter::BooleanOp) -> Self {
        match value {
            filter::BooleanOp::Equals(value) => Self::Equals(value),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DateOp {
    Before(DateTime<Utc>),
    After(DateTime<Utc>),
    Between {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
    InLast(i64),
}

impl From<DateOp> for filter::DateOp {
    fn from(value: DateOp) -> Self {
        match value {
            DateOp::Before(value) => Self::Before(value),
            DateOp::After(value) => Self::After(value),
            DateOp::Between { start, end } => Self::Between(Range { start, end }),
            DateOp::InLast(value) => Self::InLast(value),
        }
    }
}

impl From<filter::DateOp> for DateOp {
    fn from(value: filter::DateOp) -> Self {
        match value {
            filter::DateOp::Before(value) => Self::Before(value),
            filter::DateOp::After(value) => Self::After(value),
            filter::DateOp::Between(value) => Self::Between {
                start: value.start,
                end: value.end,
            },
            filter::DateOp::InLast(value) => Self::InLast(value),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ApiErrorCode {
    NotAuthenticated,
    NotAuthorized,
    NotFound,
    AlreadyExists,
    Validation,
    BadGateway,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub(crate) struct ApiError {
    pub(crate) code: ApiErrorCode,
    pub(crate) message: String,
}

impl<E: std::error::Error> From<E> for ApiError {
    fn from(value: E) -> Self {
        Self {
            code: ApiErrorCode::Unknown,
            message: value.to_string(),
        }
    }
}

impl ApiError {
    pub(crate) fn bad_credentials() -> Self {
        Self {
            code: ApiErrorCode::NotAuthenticated,
            message: "bad credentials".into(),
        }
    }

    pub(crate) fn not_authenticated() -> Self {
        Self {
            code: ApiErrorCode::NotAuthenticated,
            message: "user not authenticated".into(),
        }
    }

    pub(crate) fn forbidden() -> Self {
        Self {
            code: ApiErrorCode::NotAuthorized,
            message: "user not authorized".into(),
        }
    }

    pub(crate) fn unknown() -> Self {
        Self {
            code: ApiErrorCode::Unknown,
            message: "unknown error".into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self.code {
            ApiErrorCode::NotAuthenticated => {
                (StatusCode::UNAUTHORIZED, axum::Json(self)).into_response()
            }
            ApiErrorCode::NotAuthorized => {
                (StatusCode::FORBIDDEN, axum::Json(self)).into_response()
            }
            ApiErrorCode::NotFound => (StatusCode::NOT_FOUND, axum::Json(self)).into_response(),
            ApiErrorCode::AlreadyExists => (StatusCode::CONFLICT, axum::Json(self)).into_response(),
            ApiErrorCode::Validation => {
                (StatusCode::UNPROCESSABLE_ENTITY, axum::Json(self)).into_response()
            }
            ApiErrorCode::BadGateway => (StatusCode::BAD_GATEWAY, axum::Json(self)).into_response(),
            ApiErrorCode::Unknown => {
                (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(self)).into_response()
            }
        }
    }
}
