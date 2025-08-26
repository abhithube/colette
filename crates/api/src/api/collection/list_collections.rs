use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_handler::{CollectionCursor, Handler as _, ListCollectionsQuery};

use crate::api::{
    ApiState,
    collection::{COLLECTIONS_TAG, Collection},
    common::{ApiError, Auth, Query},
    pagination::{PAGINATION_LIMIT, Paginated, decode_cursor},
};

#[utoipa::path(
    get,
    path = "",
    params(CollectionListQuery),
    responses(OkResponse, ErrResponse),
    operation_id = "listCollections",
    description = "List user collections",
    tag = COLLECTIONS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<CollectionListQuery>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    let cursor = query
        .cursor
        .map(|e| decode_cursor::<CollectionCursor>(&e))
        .transpose()
        .map_err(|e| ErrResponse::InternalServerError(e.into()))?;

    match state
        .list_collections
        .handle(ListCollectionsQuery {
            cursor,
            limit: Some(PAGINATION_LIMIT),
            user_id: user_id.as_inner(),
        })
        .await
    {
        Ok(collections) => {
            let data = collections
                .try_into()
                .map_err(ErrResponse::InternalServerError)?;

            Ok(OkResponse(data))
        }
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub(super) struct CollectionListQuery {
    /// Pagination cursor
    #[param(nullable = false)]
    cursor: Option<String>,
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Paginated list of collections")]
pub(super) struct OkResponse(Paginated<Collection>);

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

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
