use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::subscription;

use super::SUBSCRIPTIONS_TAG;
use crate::{
    ApiState,
    common::{AuthUser, BaseError, Error, Id},
};

#[utoipa::path(
    delete,
    path = "/{id}",
    params(Id),
    responses(DeleteResponse),
    operation_id = "deleteSubscription",
    description = "Delete a subscription by ID",
    tag = SUBSCRIPTIONS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    AuthUser(user_id): AuthUser,
) -> Result<DeleteResponse, Error> {
    match state
        .subscription_service
        .delete_subscription(id, user_id)
        .await
    {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            subscription::Error::Forbidden(_) => Ok(DeleteResponse::Forbidden(BaseError {
                message: e.to_string(),
            })),
            subscription::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum DeleteResponse {
    #[response(status = 204, description = "Successfully deleted subscription")]
    NoContent,

    #[response(status = 403, description = "User not authorized")]
    Forbidden(BaseError),

    #[response(status = 404, description = "Subscription not found")]
    NotFound(BaseError),
}

impl IntoResponse for DeleteResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}
