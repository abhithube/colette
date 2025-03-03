use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::collection;

use super::{COLLECTIONS_TAG, Collection};
use crate::api::{
    ApiState,
    bookmark::BookmarkFilter,
    common::{AuthUser, BaseError, Error, Id, NonEmptyString},
};

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = CollectionUpdate,
    responses(UpdateResponse),
    operation_id = "updateCollection",
    description = "Update a collection by ID",
    tag = COLLECTIONS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<CollectionUpdate>,
) -> Result<UpdateResponse, Error> {
    match state
        .collection_service
        .update_collection(id, body.into(), user_id)
        .await
    {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
        Err(e) => match e {
            collection::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CollectionUpdate {
    #[schema(value_type = Option<String>, min_length = 1, nullable = false)]
    pub title: Option<NonEmptyString>,
    pub filter: Option<BookmarkFilter>,
}

impl From<CollectionUpdate> for collection::CollectionUpdate {
    fn from(value: CollectionUpdate) -> Self {
        Self {
            title: value.title.map(Into::into),
            filter: value.filter.map(Into::into),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated collection")]
    Ok(Collection),

    #[response(status = 404, description = "Collection not found")]
    NotFound(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for UpdateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
