use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::subscription::SubscriptionError;
use colette_handler::{Handler as _, UpdateSubscriptionCommand, UpdateSubscriptionError};

use crate::{
    ApiState,
    common::{ApiError, Auth, Id, Json, NonEmptyString, Path},
    subscription::SUBSCRIPTIONS_TAG,
};

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = SubscriptionUpdate,
    responses(OkResponse, ErrResponse),
    operation_id = "updateSubscription",
    description = "Update a subscription by ID",
    tag = SUBSCRIPTIONS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Auth { user_id }: Auth,
    Json(body): Json<SubscriptionUpdate>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .update_subscription
        .handle(UpdateSubscriptionCommand {
            id: id.into(),
            title: body.title.map(Into::into),
            description: body.description.map(|e| e.map(Into::into)),
            user_id,
        })
        .await
    {
        Ok(_) => Ok(OkResponse),
        Err(e) => match e {
            UpdateSubscriptionError::Subscription(SubscriptionError::NotFound(_)) => {
                Err(ErrResponse::NotFound(e.into()))
            }
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

/// Updates to make to an existing subscription
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct SubscriptionUpdate {
    /// Human-readable name for the subscription to update, cannot be empty
    #[schema(value_type = Option<String>, min_length = 1)]
    title: Option<NonEmptyString>,
    /// Description for the subscription to update, cannot be empty
    #[serde(default, with = "serde_with::rust::double_option")]
    #[schema(value_type = Option<Option<String>>, min_length = 1)]
    description: Option<Option<NonEmptyString>>,
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::NO_CONTENT, description = "Successfully updated subscription")]
pub(super) struct OkResponse;

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::NOT_FOUND, description = "Subscription not found")]
    NotFound(ApiError),

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
