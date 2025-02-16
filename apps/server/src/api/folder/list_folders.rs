use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use colette_core::folder;
use uuid::Uuid;

use super::{FOLDERS_TAG, Folder};
use crate::api::{
    ApiState,
    common::{AuthUser, Error, Paginated},
};

#[utoipa::path(
    get,
    path = "",
    params(FolderListQuery),
    responses(ListResponse),
    operation_id = "listFolders",
    description = "List user folders",
    tag = FOLDERS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<FolderListQuery>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse, Error> {
    match state
        .folder_service
        .list_folders(query.into(), user_id)
        .await
    {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct FolderListQuery {
    #[param(nullable = false)]
    pub filter_by_parent: Option<bool>,
    pub parent_id: Option<Uuid>,
}

impl From<FolderListQuery> for folder::FolderListQuery {
    fn from(value: FolderListQuery) -> Self {
        Self {
            parent_id: if value.filter_by_parent.unwrap_or(value.parent_id.is_some()) {
                Some(value.parent_id)
            } else {
                None
            },
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of folders")]
    Ok(Paginated<Folder>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
