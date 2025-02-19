use chrono::{DateTime, Utc};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{ApiState, common::Paginated};

mod create_folder;
mod delete_folder;
mod get_folder;
mod list_folders;
mod update_folder;

pub const FOLDERS_TAG: &str = "Folders";

#[derive(OpenApi)]
#[openapi(components(schemas(Folder, Paginated<Folder>, FolderPathItem, create_folder::FolderCreate, update_folder::FolderUpdate)))]
pub struct FolderApi;

impl FolderApi {
    pub fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(FolderApi::openapi())
            .routes(routes!(list_folders::handler, create_folder::handler))
            .routes(routes!(
                get_folder::handler,
                update_folder::handler,
                delete_folder::handler
            ))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: Uuid,
    pub title: String,
    #[schema(required)]
    pub parent_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    pub path: Vec<FolderPathItem>,
}

impl From<colette_core::Folder> for Folder {
    fn from(value: colette_core::Folder) -> Self {
        Self {
            id: value.id,
            title: value.title,
            parent_id: value.parent_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            path: value.path.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct FolderPathItem {
    pub id: Uuid,
    pub title: String,
}

impl From<colette_core::folder::FolderPathItem> for FolderPathItem {
    fn from(value: colette_core::folder::FolderPathItem) -> Self {
        Self {
            id: value.id,
            title: value.title,
        }
    }
}
