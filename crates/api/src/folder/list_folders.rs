use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use colette_core::folder::{self, FolderService};
use uuid::Uuid;

use super::Folder;
use crate::{
    Paginated,
    common::{Error, FOLDERS_TAG, Session},
};

#[derive(Clone, Debug, serde::Deserialize, utoipa::IntoParams)]
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
    State(service): State<Arc<FolderService>>,
    Query(query): Query<FolderListQuery>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service.list_folders(query.into(), session.user_id).await {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
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
