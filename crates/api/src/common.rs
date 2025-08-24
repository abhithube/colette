use std::sync::Arc;

use axum::{
    extract::{FromRequestParts, Request, State},
    http::{StatusCode, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    extract::cookie::{Cookie, SameSite},
    headers::{Authorization, HeaderMapExt, authorization::Bearer},
};
use colette_authentication::UserId;
use colette_crypto::OtpCodeGenerator;
use colette_handler::*;
use colette_http::ReqwestClient;
use colette_jwt::JwtManagerImpl;
use colette_oidc::OidcClientImpl;
use colette_queue::TokioJobProducer;
use colette_repository::*;
use colette_s3::S3ClientImpl;
use colette_smtp::SmtpClientImpl;
use url::Url;
use uuid::Uuid;

/// API config
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Server config
    pub server: ServerConfig,
    /// OIDC config
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oidc: Option<OidcConfig>,
    /// Storage bucket config
    pub s3: S3Config,
}

/// API server config
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    /// Server base URL
    pub base_url: Url,
}

/// API OIDC config
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OidcConfig {
    /// OIDC sign in button text
    pub sign_in_text: String,
}

/// API storage bucket config
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct S3Config {
    /// Base URL for the image storage bucket server
    pub image_base_url: Url,
}

#[derive(Clone, axum::extract::FromRef)]
pub struct ApiState {
    // Auth
    pub send_otp: Arc<SendOtpHandler<PostgresUserRepository, SmtpClientImpl, OtpCodeGenerator>>,
    pub verify_otp: Arc<VerifyOtpHandler<PostgresUserRepository, JwtManagerImpl>>,
    pub build_authorization_url: Option<Arc<BuildAuthorizationUrlHandler<OidcClientImpl>>>,
    pub exchange_code:
        Option<Arc<ExchangeCodeHandler<PostgresUserRepository, OidcClientImpl, JwtManagerImpl>>>,
    pub get_user: Arc<GetUserHandler<PostgresUserRepository>>,
    pub refresh_access_token:
        Arc<RefreshAccessTokenHandler<PostgresUserRepository, JwtManagerImpl>>,
    pub validate_access_token: Arc<ValidateAccessTokenHandler<JwtManagerImpl>>,

    // Backup
    pub import_backup: Arc<ImportBackupHandler<PostgresBackupRepository>>,
    pub export_backup: Arc<
        ExportBackupHandler<
            PostgresBookmarkRepository,
            PostgresSubscriptionRepository,
            PostgresTagRepository,
        >,
    >,

    // Bookmarks
    pub list_bookmarks:
        Arc<ListBookmarksHandler<PostgresBookmarkRepository, PostgresCollectionRepository>>,
    pub get_bookmark: Arc<GetBookmarkHandler<PostgresBookmarkRepository>>,
    pub create_bookmark: Arc<CreateBookmarkHandler<PostgresBookmarkRepository, TokioJobProducer>>,
    pub update_bookmark: Arc<UpdateBookmarkHandler<PostgresBookmarkRepository, TokioJobProducer>>,
    pub delete_bookmark: Arc<DeleteBookmarkHandler<PostgresBookmarkRepository, TokioJobProducer>>,
    pub scrape_bookmark: Arc<ScrapeBookmarkHandler<ReqwestClient>>,
    pub refresh_bookmark: Arc<RefreshBookmarkHandler<PostgresBookmarkRepository, ReqwestClient>>,
    pub link_bookmark_tags: Arc<LinkBookmarkTagsHandler<PostgresBookmarkRepository>>,
    pub import_bookmarks: Arc<ImportBookmarksHandler<PostgresBookmarkRepository, TokioJobProducer>>,
    pub export_bookmarks: Arc<ExportBookmarksHandler<PostgresBookmarkRepository>>,
    pub archive_thumbnail:
        Arc<ArchiveThumbnailHandler<PostgresBookmarkRepository, ReqwestClient, S3ClientImpl>>,

    // Collections
    pub list_collections: Arc<ListCollectionsHandler<PostgresCollectionRepository>>,
    pub get_collection: Arc<GetCollectionHandler<PostgresCollectionRepository>>,
    pub create_collection: Arc<CreateCollectionHandler<PostgresCollectionRepository>>,
    pub update_collection: Arc<UpdateCollectionHandler<PostgresCollectionRepository>>,
    pub delete_collection: Arc<DeleteCollectionHandler<PostgresCollectionRepository>>,

    // Entries
    pub list_entries:
        Arc<ListEntriesHandler<PostgresEntryRepository, PostgresCollectionRepository>>,
    pub get_entry: Arc<GetEntryHandler<PostgresEntryRepository>>,
    pub mark_entry_as_read: Arc<MarkEntryAsReadHandler<PostgresEntryRepository>>,
    pub mark_entry_as_unread: Arc<MarkEntryAsUnreadHandler<PostgresEntryRepository>>,

    // Feeds
    pub detect_feeds: Arc<DetectFeedsHandler<ReqwestClient>>,
    pub refresh_feed:
        Arc<RefreshFeedHandler<PostgresFeedRepository, PostgresFeedEntryRepository, ReqwestClient>>,

    // Personal access tokens
    pub list_pats: Arc<ListPatsHandler<PostgresPatRepository>>,
    pub get_pat: Arc<GetPatHandler<PostgresPatRepository>>,
    pub create_pat: Arc<CreatePatHandler<PostgresPatRepository>>,
    pub update_pat: Arc<UpdatePatHandler<PostgresPatRepository>>,
    pub delete_pat: Arc<DeletePatHandler<PostgresPatRepository>>,
    pub validate_pat: Arc<ValidatePatHandler<PostgresPatRepository>>,

    // Subscriptions
    pub list_subscriptions: Arc<ListSubscriptionsHandler<PostgresSubscriptionRepository>>,
    pub get_subscription: Arc<GetSubscriptionHandler<PostgresSubscriptionRepository>>,
    pub create_subscription: Arc<CreateSubscriptionHandler<PostgresSubscriptionRepository>>,
    pub update_subscription: Arc<UpdateSubscriptionHandler<PostgresSubscriptionRepository>>,
    pub delete_subscription: Arc<DeleteSubscriptionHandler<PostgresSubscriptionRepository>>,
    pub link_subscription_tags: Arc<LinkSubscriptionTagsHandler<PostgresSubscriptionRepository>>,
    pub import_subscriptions: Arc<ImportSubscriptionsHandler<PostgresSubscriptionRepository>>,
    pub export_subscriptions: Arc<ExportSubscriptionsHandler<PostgresSubscriptionRepository>>,

    // Tags
    pub list_tags: Arc<ListTagsHandler<PostgresTagRepository>>,
    pub get_tag: Arc<GetTagHandler<PostgresTagRepository>>,
    pub create_tag: Arc<CreateTagHandler<PostgresTagRepository>>,
    pub update_tag: Arc<UpdateTagHandler<PostgresTagRepository>>,
    pub delete_tag: Arc<DeleteTagHandler<PostgresTagRepository>>,

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

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub(crate) struct CreatedResource {
    /// Unique identifier of the resource
    pub(crate) id: Uuid,
}

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

pub(crate) async fn verify_auth_extension(
    State(state): State<ApiState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    if let Some(Authorization(bearer)) = req.headers().typed_get::<Authorization<Bearer>>() {
        let claims = state
            .validate_access_token
            .handle(ValidateAccessTokenQuery {
                access_token: bearer.token().to_string(),
            })
            .await
            .map_err(|_| ApiError::not_authenticated())?;

        req.extensions_mut().insert(Auth {
            user_id: claims.sub().parse::<Uuid>().unwrap().into(),
        });
    } else if let Some(header) = req.headers().get("X-Api-Key").and_then(|e| e.to_str().ok()) {
        let Ok(user_id) = state
            .validate_pat
            .handle(ValidatePatQuery {
                value: header.into(),
            })
            .await
        else {
            tracing::debug!("invalid PAT");

            return Err(ApiError::not_authenticated());
        };

        req.extensions_mut().insert(Auth { user_id });
    }

    Ok(next.run(req).await)
}

#[derive(Debug, Clone)]
pub(crate) struct Auth {
    pub(crate) user_id: UserId,
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

pub(crate) fn build_cookie<'a, C: Into<Cookie<'a>>>(c: C, max_age: Option<i64>) -> Cookie<'a> {
    let mut builder = Cookie::build(c)
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::None);

    if let Some(max_age) = max_age {
        builder = builder.max_age(time::Duration::seconds(max_age));
    }

    builder.build()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ApiErrorCode {
    NotAuthenticated,
    NotAuthorized,
    NotFound,
    Conflict,
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
            ApiErrorCode::Conflict => (StatusCode::CONFLICT, axum::Json(self)).into_response(),
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
