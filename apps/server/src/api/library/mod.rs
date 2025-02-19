use colette_core::library;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{ApiState, collection::Collection, common::Paginated, feed::Feed, folder::Folder};

mod list_library_items;

pub const LIBRARY_TAG: &str = "Library";

#[derive(OpenApi)]
#[openapi(components(schemas(LibraryItem, Paginated<LibraryItem>)))]
pub struct LibraryApi;

impl LibraryApi {
    pub fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(LibraryApi::openapi())
            .routes(routes!(list_library_items::handler))
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum LibraryItem {
    Folder(Folder),
    Feed(Feed),
    Collection(Collection),
}

impl From<colette_core::library::LibraryItem> for LibraryItem {
    fn from(value: colette_core::library::LibraryItem) -> Self {
        match value {
            colette_core::library::LibraryItem::Folder(folder) => Self::Folder(folder.into()),
            colette_core::library::LibraryItem::Feed(feed) => Self::Feed(feed.into()),
            colette_core::library::LibraryItem::Collection(collection) => {
                Self::Collection(collection.into())
            }
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct LibraryItemListQuery {
    pub folder_id: Option<Uuid>,
    #[param(nullable = false)]
    pub cursor: Option<String>,
}

impl From<LibraryItemListQuery> for library::LibraryItemListQuery {
    fn from(value: LibraryItemListQuery) -> Self {
        Self {
            folder_id: value.folder_id,
            cursor: value.cursor,
        }
    }
}
