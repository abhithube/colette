use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{
    Handler as _,
    collection::{CreateCollectionCommand, CreateCollectionError},
};

use crate::{
    ApiState,
    bookmark::BookmarkFilter,
    collection::COLLECTIONS_TAG,
    common::{ApiError, Auth, CreatedResource, Json, NonEmptyString},
};

#[utoipa::path(
  post,
  path = "",
  request_body = CollectionCreate,
  responses(OkResponse, ErrResponse),
  operation_id = "createCollection",
  description = "Create a collection",
  tag = COLLECTIONS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Auth { user_id }: Auth,
    Json(body): Json<CollectionCreate>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .create_collection
        .handle(CreateCollectionCommand {
            title: body.title.into(),
            filter: body.filter.into(),
            user_id,
        })
        .await
    {
        Ok(data) => Ok(OkResponse(CreatedResource {
            id: data.id.as_inner(),
        })),
        Err(e) => match e {
            CreateCollectionError::Conflict(_) => Err(ErrResponse::Conflict(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct CollectionCreate {
    #[schema(value_type = String, min_length = 1)]
    title: NonEmptyString,
    filter: BookmarkFilter,
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::CREATED, description = "New collection ID")]
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

    #[response(status = StatusCode::CONFLICT, description = "Collection already exists")]
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
