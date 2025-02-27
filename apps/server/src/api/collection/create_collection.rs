use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::collection;

use super::{BookmarkFilter, COLLECTIONS_TAG, Collection};
use crate::api::{
    ApiState,
    common::{AuthUser, BaseError, Error, NonEmptyString},
};

#[utoipa::path(
  post,
  path = "",
  request_body = CollectionCreate,
  responses(CreateResponse),
  operation_id = "createCollection",
  description = "Create a collection",
  tag = COLLECTIONS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<CollectionCreate>,
) -> Result<CreateResponse, Error> {
    match state
        .collection_service
        .create_collection(body.into(), user_id)
        .await
    {
        Ok(data) => Ok(CreateResponse::Created(data.into())),
        Err(e) => match e {
            collection::Error::Conflict(_) => Ok(CreateResponse::Conflict(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CollectionCreate {
    #[schema(value_type = String, min_length = 1)]
    pub title: NonEmptyString,
    pub filter: BookmarkFilter,
}

impl From<CollectionCreate> for collection::CollectionCreate {
    fn from(value: CollectionCreate) -> Self {
        Self {
            title: value.title.into(),
            filter: value.filter.into(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created collection")]
    Created(Collection),

    #[response(status = 409, description = "Collection already exists")]
    Conflict(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
