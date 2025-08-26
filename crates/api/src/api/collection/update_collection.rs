use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_crud::CollectionError;
use colette_handler::{Handler as _, UpdateCollectionCommand, UpdateCollectionError};

use crate::api::{
    ApiState,
    collection::{BookmarkFilter, COLLECTIONS_TAG},
    common::{ApiError, Auth, Id, Json, NonEmptyString, Path},
};

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = CollectionUpdate,
    responses(OkResponse, ErrResponse),
    operation_id = "updateCollection",
    description = "Update a collection by ID",
    tag = COLLECTIONS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Auth { user_id }: Auth,
    Json(body): Json<CollectionUpdate>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .update_collection
        .handle(UpdateCollectionCommand {
            id: id.into(),
            title: body.title.map(Into::into),
            filter: body.filter.map(Into::into),
            user_id,
        })
        .await
    {
        Ok(_) => Ok(OkResponse),
        Err(e) => match e {
            UpdateCollectionError::Collection(CollectionError::NotFound(_)) => {
                Err(ErrResponse::NotFound(e.into()))
            }
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct CollectionUpdate {
    #[schema(value_type = Option<String>, min_length = 1, nullable = false)]
    title: Option<NonEmptyString>,
    filter: Option<BookmarkFilter>,
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::NO_CONTENT, description = "Successfully updated collection")]
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

    #[response(status = StatusCode::NOT_FOUND, description = "Collection not found")]
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
