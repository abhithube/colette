use std::sync::Arc;

use colette_core::library::LibraryService;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{Paginated, bookmark::Bookmark, feed::Feed, folder::Folder};

mod list_library_items;

#[derive(Clone, axum::extract::FromRef)]
pub struct LibraryState {
    service: Arc<LibraryService>,
}

impl LibraryState {
    pub fn new(service: Arc<LibraryService>) -> Self {
        Self { service }
    }
}

#[derive(OpenApi)]
#[openapi(components(schemas(LibraryItem, Paginated<LibraryItem>)))]
pub struct LibraryApi;

impl LibraryApi {
    pub fn router() -> OpenApiRouter<LibraryState> {
        OpenApiRouter::with_openapi(LibraryApi::openapi())
            .routes(routes!(list_library_items::handler))
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum LibraryItem {
    Folder(Folder),
    Feed(Feed),
    Bookmark(Bookmark),
}

impl From<colette_core::LibraryItem> for LibraryItem {
    fn from(value: colette_core::LibraryItem) -> Self {
        match value {
            colette_core::LibraryItem::Folder(folder) => Self::Folder(folder.into()),
            colette_core::LibraryItem::Feed(feed) => Self::Feed(feed.into()),
            colette_core::LibraryItem::Bookmark(bookmark) => Self::Bookmark(bookmark.into()),
        }
    }
}
