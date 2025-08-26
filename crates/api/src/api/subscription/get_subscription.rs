use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_crud::SubscriptionError;
use colette_handler::{GetSubscriptionError, GetSubscriptionQuery, Handler as _};

use crate::api::{
    ApiState,
    common::{ApiError, Auth, Id, Path},
    subscription::{SUBSCRIPTIONS_TAG, Subscription},
};

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(OkResponse, ErrResponse),
    operation_id = "getSubscription",
    description = "Get a subscription by ID",
    tag = SUBSCRIPTIONS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state
        .get_subscription
        .handle(GetSubscriptionQuery {
            id,
            user_id: user_id.as_inner(),
        })
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => match e {
            GetSubscriptionError::Subscription(SubscriptionError::NotFound(_)) => {
                Err(ErrResponse::NotFound(e.into()))
            }
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Subscription by ID")]
pub(super) struct OkResponse(Subscription);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::NOT_FOUND, description = "Subscription not found")]
    NotFound(ApiError),

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
