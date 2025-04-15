use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::subscription;
use uuid::Uuid;

use super::SUBSCRIPTIONS_TAG;
use crate::{
    ApiState,
    common::{AuthUser, BaseError, Error, Id},
};

#[utoipa::path(
    patch,
    path = "/{id}/linkTags",
    params(Id),
    request_body = LinkSubscriptionTags,
    responses(LinkTagsResponse),
    operation_id = "linkSubscriptionTags",
    description = "Link a list of tags to a subscription",
    tag = SUBSCRIPTIONS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<LinkSubscriptionTags>,
) -> Result<LinkTagsResponse, Error> {
    match state
        .subscription_service
        .link_subscription_tags(id, body.into(), user_id)
        .await
    {
        Ok(_) => Ok(LinkTagsResponse::NoContent),
        Err(e) => match e {
            subscription::Error::Forbidden(_) => Ok(LinkTagsResponse::Forbidden(BaseError {
                message: e.to_string(),
            })),
            subscription::Error::NotFound(_) => Ok(LinkTagsResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LinkSubscriptionTags {
    pub tag_ids: Vec<Uuid>,
}

impl From<LinkSubscriptionTags> for subscription::LinkSubscriptionTags {
    fn from(value: LinkSubscriptionTags) -> Self {
        Self {
            tag_ids: value.tag_ids,
        }
    }
}

#[allow(dead_code, clippy::large_enum_variant)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum LinkTagsResponse {
    #[response(status = 200, description = "Successfully linked tags")]
    NoContent,

    #[response(status = 403, description = "User not authorized")]
    Forbidden(BaseError),

    #[response(status = 404, description = "Subscription not found")]
    NotFound(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for LinkTagsResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
