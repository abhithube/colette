use colette_core::library;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{ApiState, collection::Collection, common::Paginated, feed::Feed, folder::Folder};

mod list_collection_tree;
mod list_feed_tree;

pub const LIBRARY_TAG: &str = "Library";

#[derive(OpenApi)]
#[openapi(components(schemas(FeedTreeItem, Paginated<FeedTreeItem>, CollectionTreeItem, Paginated<CollectionTreeItem>)))]
pub struct LibraryApi;

impl LibraryApi {
    pub fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(LibraryApi::openapi())
            .routes(routes!(list_feed_tree::handler))
            .routes(routes!(list_collection_tree::handler))
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum FeedTreeItem {
    Folder(Folder),
    Feed(Feed),
}

impl From<colette_core::library::FeedTreeItem> for FeedTreeItem {
    fn from(value: colette_core::library::FeedTreeItem) -> Self {
        match value {
            colette_core::library::FeedTreeItem::Folder(folder) => Self::Folder(folder.into()),
            colette_core::library::FeedTreeItem::Feed(feed) => Self::Feed(feed.into()),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum CollectionTreeItem {
    Folder(Folder),
    Collection(Collection),
}

impl From<colette_core::library::CollectionTreeItem> for CollectionTreeItem {
    fn from(value: colette_core::library::CollectionTreeItem) -> Self {
        match value {
            colette_core::library::CollectionTreeItem::Folder(folder) => {
                Self::Folder(folder.into())
            }
            colette_core::library::CollectionTreeItem::Collection(feed) => {
                Self::Collection(feed.into())
            }
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct TreeListQuery {
    pub folder_id: Option<Uuid>,
    #[param(nullable = false)]
    pub cursor: Option<String>,
}

impl From<TreeListQuery> for library::TreeListQuery {
    fn from(value: TreeListQuery) -> Self {
        Self {
            folder_id: value.folder_id,
            cursor: value.cursor,
        }
    }
}
