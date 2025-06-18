use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::subscription;
use uuid::Uuid;

use super::SUBSCRIPTIONS_TAG;
use crate::{
    ApiState,
    common::{ApiError, Auth, Id, Json, Path},
};

#[utoipa::path(
    post,
    path = "/{id}/linkTags",
    params(Id),
    request_body = LinkSubscriptionTags,
    responses(OkResponse, ErrResponse),
    operation_id = "linkSubscriptionTags",
    description = "Link a list of tags to a subscription",
    tag = SUBSCRIPTIONS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Auth { user_id }: Auth,
    Json(body): Json<LinkSubscriptionTags>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .subscription_service
        .link_subscription_tags(id, body.into(), user_id)
        .await
    {
        Ok(_) => Ok(OkResponse),
        Err(e) => match e {
            subscription::Error::Forbidden(_) => Err(ErrResponse::Forbidden(e.into())),
            subscription::Error::NotFound(_) => Err(ErrResponse::NotFound(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

/// Action to link tags to a user subscription
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct LinkSubscriptionTags {
    /// Unique identifiers of the tags to link to the subscription
    tag_ids: Vec<Uuid>,
}

impl From<LinkSubscriptionTags> for subscription::LinkSubscriptionTags {
    fn from(value: LinkSubscriptionTags) -> Self {
        Self {
            tag_ids: value.tag_ids,
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::NO_CONTENT, description = "Successfully linked tags")]
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

    #[response(status = StatusCode::FORBIDDEN, description = "User not authorized")]
    Forbidden(ApiError),

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
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
