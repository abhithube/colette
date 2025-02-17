use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use colette_core::collection;
use uuid::Uuid;

use super::{COLLECTIONS_TAG, Collection};
use crate::api::{
    ApiState,
    common::{AuthUser, Error, Paginated},
};

#[utoipa::path(
    get,
    path = "",
    params(CollectionListQuery),
    responses(ListResponse),
    operation_id = "listCollections",
    description = "List user collections",
    tag = COLLECTIONS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<CollectionListQuery>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse, Error> {
    match state
        .collection_service
        .list_collections(query.into(), user_id)
        .await
    {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct CollectionListQuery {
    #[param(nullable = false)]
    pub filter_by_folder: Option<bool>,
    pub folder_id: Option<Uuid>,
}

impl From<CollectionListQuery> for collection::CollectionListQuery {
    fn from(value: CollectionListQuery) -> Self {
        Self {
            folder_id: if value.filter_by_folder.unwrap_or(value.folder_id.is_some()) {
                Some(value.folder_id)
            } else {
                None
            },
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of collections")]
    Ok(Paginated<Collection>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
