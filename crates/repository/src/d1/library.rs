use std::sync::Arc;

use colette_core::{
    bookmark::BookmarkFindParams,
    common::Findable,
    feed::FeedFindParams,
    library::{Error, LibraryItemFindParams, LibraryRepository},
    Folder, LibraryItem,
};
use sea_query::SqliteQueryBuilder;
use worker::D1Database;

use super::{bookmark::BookmarkSelect, feed::FeedSelect, D1Binder};

#[derive(Clone)]
pub struct D1LibraryRepository {
    db: Arc<D1Database>,
}

impl D1LibraryRepository {
    pub fn new(db: Arc<D1Database>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for D1LibraryRepository {
    type Params = LibraryItemFindParams;
    type Output = Result<Vec<LibraryItem>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let queries = vec![
            crate::folder::select(
                None,
                params.user_id,
                Some(params.folder_id),
                params.limit,
                None,
            )
            .build_d1(SqliteQueryBuilder),
            super::feed::build_select(FeedFindParams {
                folder_id: Some(params.folder_id),
                user_id: params.user_id,
                limit: params.limit,
                ..Default::default()
            })
            .build_d1(SqliteQueryBuilder),
            super::bookmark::build_select(BookmarkFindParams {
                folder_id: Some(params.folder_id),
                user_id: params.user_id,
                limit: params.limit,
                ..Default::default()
            })
            .build_d1(SqliteQueryBuilder),
        ];

        let result = super::batch(&self.db, queries)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut folders = result[0]
            .results::<Folder>()
            .map(|e| e.into_iter().map(LibraryItem::Folder).collect::<Vec<_>>())
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut feeds = result[1]
            .results::<FeedSelect>()
            .map(|e| {
                e.into_iter()
                    .map(|e| LibraryItem::Feed(e.into()))
                    .collect::<Vec<_>>()
            })
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut bookmarks = result[2]
            .results::<BookmarkSelect>()
            .map(|e| {
                e.into_iter()
                    .map(|e| LibraryItem::Bookmark(e.into()))
                    .collect::<Vec<_>>()
            })
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut library_items = Vec::new();
        library_items.append(&mut folders);
        library_items.append(&mut feeds);
        library_items.append(&mut bookmarks);

        Ok(library_items)
    }
}

impl LibraryRepository for D1LibraryRepository {}
