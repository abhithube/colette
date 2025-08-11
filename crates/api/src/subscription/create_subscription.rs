use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{
    Handler as _,
    subscription::{CreateSubscriptionCommand, CreateSubscriptionError},
};
use uuid::Uuid;

use crate::{
    ApiState,
    common::{ApiError, Auth, CreatedResource, Json, NonEmptyString},
    subscription::SUBSCRIPTIONS_TAG,
};

#[utoipa::path(
    post,
    path = "",
    request_body = SubscriptionCreate,
    responses(OkResponse, ErrResponse),
    operation_id = "createSubscription",
    description = "Subscribe to a web feed",
    tag = SUBSCRIPTIONS_TAG
  )]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Auth { user_id }: Auth,
    Json(body): Json<SubscriptionCreate>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .create_subscription
        .handle(CreateSubscriptionCommand {
            title: body.title.into(),
            description: body.description.map(Into::into),
            feed_id: body.feed_id.into(),
            user_id,
        })
        .await
    {
        Ok(data) => Ok(OkResponse(CreatedResource {
            id: data.id.as_inner(),
        })),
        Err(e) => match e {
            CreateSubscriptionError::Conflict(_) => Err(ErrResponse::Conflict(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

/// Data to create a new user subscription
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct SubscriptionCreate {
    /// Human-readable name for the new subscription, cannot be empty
    #[schema(value_type = String, min_length = 1)]
    title: NonEmptyString,
    /// Description for the new subscription, cannot be empty
    #[schema(value_type = Option<String>, min_length = 1)]
    description: Option<NonEmptyString>,
    /// Unique identifier of the associated RSS feed
    feed_id: Uuid,
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::CREATED, description = "New subscription ID")]
pub(super) struct OkResponse(CreatedResource);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, axum::Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::CONFLICT, description = "Subscription already exists")]
    Conflict(ApiError),

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
