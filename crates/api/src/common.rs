use std::{net::SocketAddr, ops::Range, sync::Arc};

use axum::{
    extract::{ConnectInfo, FromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{TypedHeader, extract::CookieJar, headers::UserAgent};
use chrono::{DateTime, Utc};
use colette_auth::AuthAdapter;
use colette_core::{
    api_key::ApiKeyService, bookmark::BookmarkService, collection::CollectionService, common,
    feed::FeedService, feed_entry::FeedEntryService, filter, job::JobService,
    stream::StreamService, subscription::SubscriptionService,
    subscription_entry::SubscriptionEntryService, tag::TagService,
};
use torii::{SessionToken, Torii, User, UserId};
use url::Url;
use uuid::Uuid;

pub(crate) const SESSION_COOKIE: &str = "session_id";

#[derive(Clone, axum::extract::FromRef)]
pub struct ApiState {
    pub auth: Arc<Torii<AuthAdapter>>,
    pub api_key_service: Arc<ApiKeyService>,
    pub bookmark_service: Arc<BookmarkService>,
    pub collection_service: Arc<CollectionService>,
    pub feed_service: Arc<FeedService>,
    pub feed_entry_service: Arc<FeedEntryService>,
    pub job_service: Arc<JobService>,
    pub stream_service: Arc<StreamService>,
    pub subscription_service: Arc<SubscriptionService>,
    pub subscription_entry_service: Arc<SubscriptionEntryService>,
    pub tag_service: Arc<TagService>,
    pub image_base_url: Url,
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
pub(crate) struct Id(pub(crate) Uuid);

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

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Paginated<T: utoipa::ToSchema> {
    pub(crate) data: Vec<T>,
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

#[derive(Clone, Debug)]
pub(crate) struct ConnectionInfo {
    pub(crate) ip_address: Option<String>,
    pub(crate) user_agent: Option<String>,
}

pub(crate) async fn add_connection_info_extension(
    user_agent: Option<TypedHeader<UserAgent>>,
    addr: ConnectInfo<SocketAddr>,
    mut request: Request,
    next: Next,
) -> Response {
    request.extensions_mut().insert(ConnectionInfo {
        ip_address: Some(addr.ip().to_string()),
        user_agent: user_agent.map(|e| e.to_string()),
    });

    next.run(request).await
}

pub(crate) async fn add_user_extension(
    State(state): State<ApiState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    req.extensions_mut().insert(None::<User>);

    let session_id = jar
        .get(SESSION_COOKIE)
        .and_then(|cookie| cookie.value().parse::<String>().ok());

    if let Some(session_id) = session_id {
        let Ok(session) = state
            .auth
            .get_session(&SessionToken::new(&session_id))
            .await
        else {
            tracing::debug!("session not found");

            return Err(ApiError::unauthenticated());
        };

        let Ok(user) = state.auth.get_user(&session.user_id).await else {
            tracing::debug!("user not found");

            return Err(ApiError::unauthenticated());
        };

        req.extensions_mut().insert(user.clone());
        if let Some(valid_user) = user {
            req.extensions_mut().insert(valid_user);
        }
    } else {
        let header = req.headers().get("X-Api-Key");

        if let Some(header) = header {
            let Ok(header) = header.to_str() else {
                tracing::debug!("invalid header");

                return Err(ApiError::unauthenticated());
            };

            let Ok(api_key) = state.api_key_service.validate_api_key(header.into()).await else {
                tracing::debug!("invalid API key");

                return Err(ApiError::unauthenticated());
            };

            let Ok(user) = state.auth.get_user(&UserId::new(&api_key.user_id)).await else {
                tracing::debug!("user not found");

                return Err(ApiError::unauthenticated());
            };

            req.extensions_mut().insert(user.clone());
            if let Some(valid_user) = user {
                req.extensions_mut().insert(valid_user);
            }
        }
    }

    Ok(next.run(req).await)
}

#[derive(Debug, Clone)]
pub(crate) struct AuthUser(pub(crate) String);

impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<User>()
            .cloned()
            .map(|e| AuthUser(e.id.into_inner()))
            .ok_or_else(|| {
                tracing::debug!("failed to extract authenticated user");

                ApiError::unauthenticated()
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
#[schema(title = "Error")]
pub(crate) struct ApiError {
    pub(crate) message: String,
}

impl<E: std::error::Error> From<E> for ApiError {
    fn from(value: E) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl ApiError {
    pub(crate) fn bad_credentials() -> Self {
        Self {
            message: "Bad credentials".into(),
        }
    }

    pub(crate) fn unauthenticated() -> Self {
        Self {
            message: "user not authenticated".into(),
        }
    }

    pub(crate) fn unknown() -> Self {
        Self {
            message: "unknown error".into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        axum::Json(self).into_response()
    }
}
