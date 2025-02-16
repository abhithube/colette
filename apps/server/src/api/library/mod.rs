use std::sync::Arc;

use colette_core::library::LibraryService;
use url::Url;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::{bookmark::Bookmark, common::Paginated, feed::Feed, folder::Folder};

mod list_library_items;

pub const LIBRARY_TAG: &str = "Library";

#[derive(OpenApi)]
#[openapi(components(schemas(LibraryItem, Paginated<LibraryItem>)))]
pub struct LibraryApi;

impl LibraryApi {
    pub fn router() -> OpenApiRouter<LibraryState> {
        OpenApiRouter::with_openapi(LibraryApi::openapi())
            .routes(routes!(list_library_items::handler))
    }
}

#[derive(Clone, axum::extract::FromRef)]
pub struct LibraryState {
    pub service: Arc<LibraryService>,
    pub bucket_url: Url,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum LibraryItem {
    Folder(Folder),
    Feed(Feed),
    Bookmark(Bookmark),
}

impl From<(colette_core::LibraryItem, Url)> for LibraryItem {
    fn from((value, bucket_url): (colette_core::LibraryItem, Url)) -> Self {
        match value {
            colette_core::LibraryItem::Folder(folder) => Self::Folder(folder.into()),
            colette_core::LibraryItem::Feed(feed) => Self::Feed(feed.into()),
            colette_core::LibraryItem::Bookmark(bookmark) => {
                Self::Bookmark((bookmark, bucket_url).into())
            }
        }
    }
}
