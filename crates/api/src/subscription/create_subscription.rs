use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::subscription;
use uuid::Uuid;

use super::{SUBSCRIPTIONS_TAG, Subscription};
use crate::{
    ApiState,
    common::{AuthUser, BaseError, Error, NonEmptyString},
};

#[utoipa::path(
    post,
    path = "",
    request_body = SubscriptionCreate,
    responses(CreateResponse),
    operation_id = "createSubscription",
    description = "Subscribe to a web feed",
    tag = SUBSCRIPTIONS_TAG
  )]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<SubscriptionCreate>,
) -> Result<CreateResponse, Error> {
    match state
        .subscription_service
        .create_subscription(body.into(), user_id)
        .await
    {
        Ok(data) => Ok(CreateResponse::Created(data.into())),
        Err(e) => match e {
            subscription::Error::Conflict(_) => Ok(CreateResponse::Conflict(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionCreate {
    #[schema(value_type = String, min_length = 1)]
    pub title: NonEmptyString,
    #[schema(value_type = Option<String>, min_length = 1)]
    pub description: Option<NonEmptyString>,
    pub feed_id: Uuid,
    #[schema(nullable = false)]
    pub tags: Option<Vec<Uuid>>,
}

impl From<SubscriptionCreate> for subscription::SubscriptionCreate {
    fn from(value: SubscriptionCreate) -> Self {
        Self {
            title: value.title.into(),
            description: value.description.map(Into::into),
            feed_id: value.feed_id,
            tags: value.tags,
        }
    }
}

#[allow(dead_code, clippy::large_enum_variant)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created subscription")]
    Created(Subscription),

    #[response(status = 409, description = "Feed not cached")]
    Conflict(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::Conflict(data) => (StatusCode::CONFLICT, Json(data)).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
