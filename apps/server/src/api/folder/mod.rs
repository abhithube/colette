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
#[openapi(components(schemas(Folder, Paginated<Folder>, create_folder::FolderCreate, update_folder::FolderUpdate)))]
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
    pub folder_type: FolderType,
    #[schema(required)]
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum FolderType {
    Feeds,
    Collections,
}

impl From<FolderType> for colette_core::folder::FolderType {
    fn from(value: FolderType) -> Self {
        match value {
            FolderType::Feeds => colette_core::folder::FolderType::Feeds,
            FolderType::Collections => colette_core::folder::FolderType::Collections,
        }
    }
}

impl From<colette_core::folder::FolderType> for FolderType {
    fn from(value: colette_core::folder::FolderType) -> Self {
        match value {
            colette_core::folder::FolderType::Feeds => FolderType::Feeds,
            colette_core::folder::FolderType::Collections => FolderType::Collections,
        }
    }
}

impl From<colette_core::Folder> for Folder {
    fn from(value: colette_core::Folder) -> Self {
        Self {
            id: value.id,
            title: value.title,
            folder_type: value.folder_type.into(),
            parent_id: value.parent_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
